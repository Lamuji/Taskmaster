/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   reload.rs                                          :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: ramzi <ramzi@student.42.fr>                +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2024/06/19 16:44:11 by ramzi             #+#    #+#             */
/*   Updated: 2024/06/21 00:59:22 by ramzi            ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */


use std::sync::{Arc, Mutex};

use crate::daemon::taskmasterd::{initconfig::{get_config, Procs, Status}, server::bidirmsg::BidirectionalMessage};

use super::{start_process, stop::stop_program_internal};

pub fn handle_reload(args: Vec<String>, channel: BidirectionalMessage, procs: &mut Procs) {
    let new_config = get_config();
    let mut programs_to_start = Vec::new();

    // Arrêter et supprimer les programmes qui ne sont plus dans la nouvelle configuration
    let old_programs: Vec<String> = procs.config.programs.keys().cloned().collect();
    for name in old_programs {
        if !new_config.programs.contains_key(&name) {
            let _ = stop_program_internal(vec![name.clone()], procs);
            procs.status.retain(|s| s.lock().unwrap().name != name);
            procs.processes.lock().unwrap().remove(&name);
        }
    }

    // Démarrer les nouveaux programmes et ceux qui ont autostart changé à true
    for (name, program) in &new_config.programs {
        if !procs.config.programs.contains_key(name) || 
           (procs.config.programs[name].autostart == false && program.autostart) {
            if program.autostart {
                programs_to_start.push((name.clone(), program.clone()));
            }
        }
    }

    // Mettre à jour la configuration globale
    procs.config = new_config;

    // Démarrer les programmes identifiés
    for (name, program) in programs_to_start {
        let status = Arc::new(Mutex::new(Status::new(name.clone(), String::from("starting"))));
        procs.status.push(status.clone());
        let _ = start_process(name, program, status, procs.processes.clone());
    }

    channel.answer(String::from("Configuration reloaded")).unwrap();
}