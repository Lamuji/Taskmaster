/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   start.rs                                           :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: ramzi <ramzi@student.42.fr>                +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2024/06/19 16:45:54 by ramzi             #+#    #+#             */
/*   Updated: 2024/06/21 00:59:12 by ramzi            ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

use std::sync::{Arc, Mutex};

use crate::daemon::taskmasterd::{initconfig::{Procs, Status}, server::bidirmsg::BidirectionalMessage};

use super::{is_program_running, start_process};

pub fn handle_start(args: Vec<String>, channel: BidirectionalMessage, procs: &mut Procs) {
    if args.is_empty() {
        if let Err(e) = channel.answer("No program specified to start".to_string()) {
            eprintln!("Failed to send start response: {:?}", e);
        }
        return;
    }

    let mut response = String::new();
    for arg in args {
        if let Some(program) = procs.config.programs.get(&arg) {
            if is_program_running(arg.clone(), procs) {
                response.push_str(&format!("Program {} is already running.\n", arg));
            } else {
                let status = Arc::new(Mutex::new(Status::new(arg.clone(), String::from("starting"))));
                procs.status.retain(|s| s.lock().unwrap().name != arg);  // Remove any existing status with the same name
                procs.status.push(status.clone());
                if let Err(e) = start_process(arg.clone(), program.clone(), status.clone(), procs.processes.clone()) {
                    response.push_str(&format!("Failed to start program {}: {:?}\n", arg, e));
                } else {
                    response.push_str(&format!("Program {} started\n", arg));
                }
            }
        } else {
            response.push_str(&format!("Program {} not found in configuration\n", arg));
        }
    }
    if let Err(e) = channel.answer(response) {
        eprintln!("Failed to send start response: {:?}", e);
    }
}
