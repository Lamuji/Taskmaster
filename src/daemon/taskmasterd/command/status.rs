/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   status.rs                                          :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: ramzi <ramzi@student.42.fr>                +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2024/06/19 16:43:38 by ramzi             #+#    #+#             */
/*   Updated: 2024/06/22 15:55:01 by ramzi            ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

use crate::daemon::taskmasterd::{initconfig::Procs, server::bidirmsg::BidirectionalMessage};

pub fn handle_status(args: Vec<String>, channel: BidirectionalMessage, procs: &Procs) {
    let mut status_message = String::new();
    let processes_guard = procs.processes.lock().unwrap();

    if processes_guard.is_empty() {
        status_message.push_str("Nothing to display");
    } else {
        for (name, status) in processes_guard.iter() {
            let (state, pid_str, start_time_str) = {
                let status_guard = status.lock().unwrap();

                let pid_str = status_guard.child
                    .as_ref()
                    .map_or("N/A".to_string(), |child| child.lock().unwrap().id().to_string());

                let start_time_str = status_guard.start_time
                    .map_or("N/A".to_string(), |t| t.format("%Y-%m-%d %H:%M:%S").to_string());

                (status_guard.state.clone(), pid_str, start_time_str)
            };

            status_message.push_str(&format!(
                "\n{}       {}       {}      {}\n",
                name, pid_str, state, start_time_str
            ));
        }
    }

    if let Err(e) = channel.answer(status_message) {
        eprintln!("Failed to send status response: {:?}", e);
    }
}

