/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   restart.rs                                         :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: ramzi <ramzi@student.42.fr>                +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2024/06/19 16:44:23 by ramzi             #+#    #+#             */
/*   Updated: 2024/06/21 00:59:18 by ramzi            ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

use std::sync::{Arc, Mutex};

use crate::daemon::taskmasterd::{initconfig::{Procs, Status}, server::bidirmsg::BidirectionalMessage};

use super::{start_process, stop::stop_program_internal};

pub fn handle_restart(args: Vec<String>, channel: BidirectionalMessage, procs: &mut Procs) {
    if args.is_empty() {
        channel.answer("No program specified to restart".to_string()).unwrap();
        return;
    }

    let mut response = String::new();

    // First, stop the programs and capture the list of successfully STOPPED programs
    for arg in args.clone() {
        let stop_response = stop_program_internal(vec![arg.clone()], procs);
        if !stop_response.is_empty() {
            response.push_str(&stop_response);
        }
    }
    for arg in args {
        if let Some(program) = procs.config.programs.get(&arg) {
            let status = Arc::new(Mutex::new(Status::new(arg.clone(), String::from("starting"))));
            procs.status.retain(|s| s.lock().unwrap().name != arg);  // Remove any existing status with the same name
            procs.status.push(status.clone());
            if let Err(e) = start_process(arg.clone(), program.clone(), status.clone(), procs.processes.clone()) {
                response.push_str(&format!("Failed to restart program {}: {:?}\n", arg, e));
            } else {
                response.push_str(&format!("Program {} restarted\n", arg));
            }
        } else {
            response.push_str(&format!("Program {} not found in configuration\n", arg));
        }
    }

    if let Err(e) = channel.answer(response) {
        eprintln!("Failed to send restart response: {:?}", e);
    }
}