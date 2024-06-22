/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   stop.rs                                            :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: ramzi <ramzi@student.42.fr>                +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2024/06/19 16:46:03 by ramzi             #+#    #+#             */
/*   Updated: 2024/06/21 01:17:18 by ramzi            ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

use std::{process, thread, time::{Duration, SystemTime}};

use crate::daemon::taskmasterd::{command::{LOGFILE, SOCK_PATH}, initconfig::Procs, server::{bidirmsg::BidirectionalMessage, logfile::SaveLog}};

use super::system_time;


pub fn handle_stop(args: Vec<String>, channel: BidirectionalMessage, procs: &mut Procs) {
    if args.is_empty() {
        shutdown_daemon(channel, procs);
        return;
    }

    let response = stop_program_internal(args, procs);
    channel.answer(response).unwrap();
}

pub fn stop_program_internal(args: Vec<String>, procs: &mut Procs) -> String {
    let mut response = String::new();
    let processes = procs.processes.clone();

    for arg in args {
        let statuses: Vec<_> = {
            let processes_guard = processes.lock().unwrap();
            processes_guard
                .iter()
                .filter(|(key, _)| key.starts_with(&arg))
                .map(|(key, status)| (key.clone(), status.clone()))
                .collect()
        };

        if statuses.is_empty() {
            response.push_str(&format!("Program {} is not running\n", arg));
            continue;
        }

        for (instance_name, status) in statuses {
            let child_opt = {
                let mut status_guard = status.lock().unwrap();
                let child_opt = status_guard.child.take();
                status_guard.state = String::from("STOPPED");
                status_guard.start_time = Some(system_time(SystemTime::now()));
                child_opt
            };

            if let Some(child_arc) = child_opt {
                let mut attempts = 0;
                let max_attempts = 10;
                let mut locked = false;

                while attempts < max_attempts {
                    match child_arc.try_lock() {
                        Ok(mut child) => {
                            match child.kill() {
                                Ok(_) => {
                                    let STOPPED = child.wait().is_ok();
                                    if STOPPED {
                                        response.push_str(&format!("Program {} stopped.\n", instance_name));
                                    } else {
                                        response.push_str(&format!("Failed to stop program {}: still running.\n", instance_name));
                                    }
                                    locked = true;
                                    break;
                                },
                                Err(e) => {
                                    response.push_str(&format!("Failed to stop program {}: {}\n", instance_name, e));
                                    locked = true;
                                    break;
                                },
                            }
                        },
                        Err(_) => {
                            attempts += 1;
                            thread::sleep(Duration::from_millis(100));
                        },
                    }
                }

                if !locked {
                    response.push_str(&format!("Failed to acquire lock to stop program {}\n", instance_name));
                }
            } else {
                response.push_str(&format!("Program {} is not running\n", instance_name));
            }
        }
    }
    response
}



fn shutdown_daemon(channel: BidirectionalMessage, procs: &mut Procs) {
    let exit_msg = "Daemon shutting down...";
    channel.answer(String::from("Quit")).expect("Error when channel.answer is used");
    exit_msg.logs(LOGFILE, "Daemon");
    println!("{}", exit_msg);
    let processes = procs.processes.clone();
    let processes_guard = processes.lock().unwrap();

    for (name, status) in processes_guard.iter() {
        let child_opt = {
            let status_guard = status.lock().unwrap();
            status_guard.child.clone()
        };
        if let Some(child_arc) = child_opt {
            let mut attempts = 0;
            let max_attempts = 10;
            let mut locked = false;

            while attempts < max_attempts {
                match child_arc.try_lock() {
                    Ok(mut child) => {
                        //println!("Successfully locked child_arc for program: {}", name);
                        let _ = child.kill();
                        let _ = child.wait();
                        locked = true;
                        break;
                    },
                    Err(_) => {
                        attempts += 1;
                        thread::sleep(Duration::from_millis(100));
                    },
                }
            }
            if !locked {
                eprintln!("Failed to acquire lock to stop program {}", name);
            }
        }
    }

    if std::fs::metadata(SOCK_PATH).is_ok() {
        std::fs::remove_file(SOCK_PATH).unwrap();
    }
    thread::sleep(Duration::from_secs(2));
    println!("Daemon Exit");
    process::exit(0);
}