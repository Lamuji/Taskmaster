/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   command.rs                                         :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: ramzi <ramzi@student.42.fr>                +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2024/05/26 05:06:43 by jbettini          #+#    #+#             */
/*   Updated: 2024/07/12 23:04:57 by ramzi            ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */


pub mod stop;
pub mod start;
pub mod restart;
pub mod reload;
pub mod status;


const SOCK_PATH: &'static str = "/home/ramzi/Desktop/Taskmaster/confs/mysocket.sock";
const LOGFILE: &'static str = "/home/ramzi/Desktop/Taskmaster/confs/logfile";

use std::{collections::HashMap, fs::File, process::{self, Stdio}, sync::{mpsc::{self, Receiver, Sender}, Arc, Mutex}, thread, time::{Duration, SystemTime}};


use crate::daemon::taskmasterd::{initconfig::{Procs, Status}, server::bidirmsg::BidirectionalMessage};
use chrono::{DateTime, Local, Utc};
use libc::{umask, mode_t};
use nix::libc;
use reload::handle_reload;
use restart::handle_restart;
use serde::{Serialize, Deserialize};
use start::handle_start;
use status::handle_status;
use stop::handle_stop;

use super::{initconfig::{checker::ToUmask, get_config, parsing::ProgramConfig}, server::{self, logfile::SaveLog}};


#[derive(Debug, Serialize, Deserialize)]
pub struct Command {
    pub cmd: String,
    pub args: Vec<String>,
}

impl Command {
    pub fn new () -> Self {
        Command {
            cmd: String::new(),
            args: Vec::new(),
        }
    }
}

pub fn is_program_running(name: String, procs: &Procs) -> bool {
    let processes_guard = procs.processes.lock().unwrap();
    processes_guard
        .iter()
        .any(|(key, status)| {
            key.starts_with(&name) && status.lock().unwrap().state == "RUNNING"
        })
}


pub fn system_time(time: SystemTime) -> DateTime<Local> {
    let datetime: DateTime<Utc> = time.into();
    datetime.with_timezone(&Local)
}


fn start_process(
    name: String,
    program_config: ProgramConfig,
    status: Arc<Mutex<Status>>,
    processes: Arc<Mutex<HashMap<String, Arc<Mutex<Status>>>>>,
) -> Result<(), Box<dyn std::error::Error>> {
    for i in 0..program_config.numprocs {
        let instance_name = format!("{}_{}", name, i + 1);
        let status_clone = Arc::new(Mutex::new(Status::new(instance_name.clone(), String::from("starting"))));
        let mut program_config_clone = program_config.clone();
        let processes_clone = processes.clone();
        let startretries = program_config.startretries;
        let starttime = program_config.starttime;

        thread::spawn(move || {
            let mut attempts = 1;
            let mut started = false;

             // Convert the umask string to a Umask struct
             let  umask_struct = program_config_clone.umask.to_umask();

             // Set the umask
             let old_umask = unsafe { umask((umask_struct.owner << 6) | (umask_struct.group << 3) | umask_struct.others) };
 
             // Rest of the function...


            while attempts <= startretries {
                let start_time = SystemTime::now();
                let stdout_file = match File::create(&program_config_clone.stdout) {
                    Ok(file) => file,
                    Err(e) => {
                        eprintln!("Failed to create stdout file for program {}: {:?}", instance_name, e);
                        return;
                    }
                };

                let child = match process::Command::new(&program_config_clone.cmd)
                    .args(&program_config_clone.args)
                    .current_dir(&program_config_clone.workingdir)
                    .stdout(Stdio::from(stdout_file))
                    .spawn()
                {
                    Ok(child) => child,
                    Err(e) => {
                        eprintln!("Failed to start process for program {}: {:?}", instance_name, e);
                        attempts += 1;
                        thread::sleep(Duration::from_secs(starttime.into()));
                        continue;
                    }
                };

                {
                    let mut status_guard = status_clone.lock().unwrap();
                    status_guard.state = String::from("RUNNING");
                    status_guard.start_time = Some(system_time(start_time));
                    status_guard.child = Some(Arc::new(Mutex::new(child)));
                }

                {
                    let mut processes_guard = processes_clone.lock().unwrap();
                    processes_guard.insert(instance_name.clone(), status_clone.clone());
                }
                started = true;

                // Vérifiez si le processus reste en état RUNNING pour la durée de starttime
                thread::sleep(Duration::from_secs(starttime.into()));
                let status_guard = status_clone.lock().unwrap();
                if status_guard.state == "RUNNING" {
                    eprintln!("Process {} started successfully", instance_name);
                } else {
                    eprintln!("Process {} failed to start within the specified starttime", instance_name);
                    started = false;
                }

                break;
            }

            if !started {
                eprintln!("Failed to start program {} after {} attempts", instance_name, startretries);
            }

            loop {
                let status_guard = status_clone.lock().unwrap();
                if status_guard.state == "STOPPED" {
                    break;
                }
                drop(status_guard);
                thread::sleep(Duration::from_secs(1));
            }
            unsafe { umask(old_umask) };
        });
        // Restore the umask
    }
    Ok(())
}


pub fn load_config(procs: &mut Procs) {
    procs.config = get_config();
    for (name, program) in &procs.config.programs {
        if program.autostart {
            let status = Arc::new(Mutex::new(Status::new(name.clone(), String::from("starting"))));
            procs.status.push(status.clone());
            start_process(name.clone(), program.clone(), status, procs.processes.clone());
        }
    }
}

fn remove_last_suffix(s: &str) -> &str {
    match s.rfind('_') {
        Some(index) => &s[..index],
        None => s,
    }
}



fn check_process_status(procs: Arc<Mutex<Procs>>) {
    let mut retry_counts: HashMap<String, u32> = HashMap::new(); // Pour garder une trace des tentatives de redémarrage

    loop {
        let mut to_remove = vec![]; // Liste des processus à supprimer
        {
            let processes = procs.lock().unwrap();
            let processes_guard = processes.processes.lock().unwrap();
            for (name, status) in processes_guard.iter() {
                let child_status = {
                    let status_guard = status.lock().unwrap();
                    status_guard.child.clone()
                };

                if let Some(child_arc) = child_status {
                    let mut child = child_arc.lock().unwrap();
                    if child.try_wait().unwrap().is_some() {
                        let mut status_guard = status.lock().unwrap();
                        status_guard.state = String::from("STOPPED");

                        let basename = remove_last_suffix(&name);
                        if let Some(program) = processes.config.programs.get(basename) {
                            let retry_count = retry_counts.entry(name.clone()).or_insert(0);

                            match program.autorestart.as_str() {
                                "true" => {
                                    if *retry_count < program.startretries {
                                        eprintln!("Process {} exited, restarting...", name);
                                        // Redémarrer le processus
                                        if let Err(e) = start_process(basename.to_string(), program.clone(), status.clone(), processes.processes.clone()) {
                                            eprintln!("Failed to restart process {}: {:?}", name, e);
                                        }
                                        *retry_count += 1;
                                    } else {
                                        eprintln!("Process {} reached max restart attempts", name);
                                        retry_counts.remove(name);
                                        to_remove.push(name.clone()); // Ajouter à la liste de suppression
                                    }
                                }
                                "unexpected" => {
                                    let exit_code = child.wait().unwrap().code().unwrap_or(1);
                                    if !program.exitcodes.contains(&exit_code) {
                                        if *retry_count < program.startretries {
                                            eprintln!("Process {} exited unexpectedly with code {}, restarting...", name, exit_code);
                                            // Redémarrer le processus
                                            if let Err(e) = start_process(basename.to_string(), program.clone(), status.clone(), processes.processes.clone()) {
                                                eprintln!("Failed to restart process {}: {:?}", name, e);
                                            }
                                            *retry_count += 1;
                                        } else {
                                            eprintln!("Process {} reached max restart attempts", name);
                                            retry_counts.remove(name);
                                            to_remove.push(name.clone()); // Ajouter à la liste de suppression
                                        }
                                    }
                                }
                                _ => {
                                    retry_counts.remove(name);
                                    to_remove.push(name.clone()); // Ajouter à la liste de suppression
                                }
                            }
                            thread::sleep(Duration::from_secs(program.starttime as u64));
                        }
                    }
                }
            }
        }

        // Supprimer les processus de la liste de surveillance en dehors de la boucle
        if !to_remove.is_empty() {
            let processes = procs.lock().unwrap();
            let mut processes_guard = processes.processes.lock().unwrap();
            for name in to_remove {
                processes_guard.remove(&name);
            }
        }

        thread::sleep(Duration::from_secs(5));
    }
}

pub fn main_process() {
    "Daemon is Up".logs(LOGFILE, "Daemon");
    if std::fs::metadata(SOCK_PATH).is_ok() {
        println!("A socket is already present. Delete with \"rm -rf {}\" before starting", SOCK_PATH);
        std::process::exit(0);
    }
    let (talk_to_daemon, rec_in_daemon): (Sender<BidirectionalMessage>, Receiver<BidirectionalMessage>) = mpsc::channel();
    thread::spawn(move || server::launch_server(talk_to_daemon.clone()));

    let mut procs = Procs::new();
    let procs_arc = Arc::new(Mutex::new(procs));

    // Charger la configuration initiale et démarrer les processus
    load_config(&mut procs_arc.lock().unwrap());

    // Lancer le thread de surveillance
    let procs_clone = Arc::clone(&procs_arc);
    thread::spawn(move || check_process_status(procs_clone));

    for receive in rec_in_daemon {
        let command: Command = serde_yaml::from_str(&(receive.message.to_string())).expect("Error when parsing command");
        match command.cmd.as_str() {
            "start" => handle_start(command.args, receive, &mut procs_arc.lock().unwrap()),
            "stop" => handle_stop(command.args, receive, &mut procs_arc.lock().unwrap()),
            "restart" => handle_restart(command.args, receive, &mut procs_arc.lock().unwrap()),
            "status" => handle_status(command.args, receive, &procs_arc.lock().unwrap()),
            "reload" => handle_reload(command.args, receive, &mut procs_arc.lock().unwrap()),
            _ => panic!("Unknown command: Parsing error"),
        }
    }
}

