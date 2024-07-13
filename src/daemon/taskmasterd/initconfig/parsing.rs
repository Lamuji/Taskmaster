/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   parsing.rs                                         :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: ramzi <ramzi@student.42.fr>                +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2024/05/17 03:29:48 by jbettini          #+#    #+#             */
/*   Updated: 2024/07/12 22:58:19 by ramzi            ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

// use std::collections::HashMap;
// use serde::{Serialize, Deserialize};
// use std::fs::File;
// use std::io::Read;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone)]

pub struct ProgramConfig {
    pub cmd: String,

    pub args: Vec<String>,

    #[serde(default = "default_numprocs")]
    pub numprocs: u32,
    
    #[serde(default = "default_umask")]
    pub umask: String,
    
    #[serde(default = "default_workingdir")]
    pub workingdir: String,
    
    #[serde(default = "default_autostart")]
    pub autostart: bool,
    
    #[serde(default = "default_autorestart")]
    pub autorestart: String,
    
    #[serde(default = "default_exitcodes")]
    pub exitcodes: Vec<i32>,

    #[serde(default = "default_startretries")]
    pub startretries: u32,

    #[serde(default = "default_starttime")]
    pub starttime: u32,
    
    #[serde(default = "default_stopsignal")]
    pub stopsignal: String,
    
    #[serde(default = "default_stoptime")]
    pub stoptime: u32,
    
    pub stdout: String,
    pub stderr: String,
    pub env: Option<HashMap<String, String>>,
}

fn default_numprocs() -> u32 {
    1
}
fn default_umask() -> String {
    String::from("022")
}
fn default_workingdir() -> String {
    String::from("/tmp")
}
fn default_autostart() -> bool {
    true
}
fn default_autorestart() -> String {
    String::from("unexpected")
}
fn default_exitcodes() -> Vec<i32> {
    vec![0]
}
fn default_startretries() -> u32 {
    3
}
fn default_starttime() -> u32 {
    1
}
fn default_stopsignal() -> String {
    String::from("TERM")
}
fn default_stoptime() -> u32 {
    10
}
