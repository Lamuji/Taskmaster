/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   initconfig.rs                                      :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: ramzi <ramzi@student.42.fr>                +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2024/06/04 01:03:01 by jbettini          #+#    #+#             */
/*   Updated: 2024/06/23 17:22:34 by ramzi            ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

pub mod parsing;
pub mod checker;

// use umask::Umask;
// use umask::ToUmask;

use checker::{Schecker, Uchecker};
use chrono::{DateTime, Local};
use parsing::ProgramConfig;
use std::{collections::HashMap, process, sync::{Arc, Mutex}, time::SystemTime};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub programs: HashMap<String, ProgramConfig>,
}
#[derive(Debug)]
pub struct Procs {
    pub config: Config,
    pub status: Vec<Arc<Mutex<Status>>>,
    pub pids: HashMap<String, u32>,
    pub processes: Arc<Mutex<HashMap<String, Arc<Mutex<Status>>>>>, // Ajouté
}

impl Procs {
    pub fn new() -> Self {
        Procs {
            config: get_config(),
            status: Vec::new(),
            pids: HashMap::new(),
            processes: Arc::new(Mutex::new(HashMap::new())), // Initialisation
        }
    }
}
#[derive(Debug)]
pub struct Status {
    pub name: String,
    pub state: String,
    pub start_time: Option<DateTime<Local>>,
    pub child: Option<Arc<Mutex<process::Child>>>,
    pub pid: Option<u32>,
    pub starttime: Option<u32>
}

impl Status {
    pub fn new (name: String, state: String) -> Self {
        Status {
            name,
            state,
            start_time: None,
            child: None,
            pid: None,
            starttime: None
        }
    }
}


fn check_config(config: & mut Config) {
    for prog in config.programs.values_mut() {
        prog.cmd.trim_assign();
        prog.workingdir.trim_assign();
        prog.umask.check_umask();
        prog.autorestart.check_autorestart();
        prog.stopsignal.check_stopsignal();
        prog.numprocs.u32_field_checker();
        prog.startretries.u32_field_checker();
        prog.stoptime.u32_field_checker();
    }
}

pub fn get_config() -> Config {
    let yaml_path =  "/home/ramzi/Desktop/Taskmaster/confs/taskmaster_confs.yaml";
    let yaml = std::fs::read_to_string(yaml_path).expect("Failed to read YAML file");
    let mut config = serde_yaml::from_str(&yaml).expect("Failed to parse YAML : \n");
    check_config(& mut config);
    config
}