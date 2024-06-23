/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   checker.rs                                         :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: ramzi <ramzi@student.42.fr>                +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2024/06/05 22:12:50 by jbettini          #+#    #+#             */
/*   Updated: 2024/06/23 15:13:32 by ramzi            ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

use core::panic;
use std::vec::Vec;

const SIGNAL_NAMES: [&'static str; 33] = [
    "NONE",
    "HUP",
    "INT",
    "QUIT",
    "ILL",
    "TRAP",
    "ABRT",
    "EMT",
    "FPE",
    "KILL",
    "BUS",
    "SEGV",
    "SYS",
    "PIPE",
    "ALRM",
    "TERM",
    "URG",
    "STOP",
    "TSTP",
    "CONT",
    "CHLD",
    "TTIN",
    "TTOU",
    "IO",
    "XCPU",
    "XFSZ",
    "VTALRM",
    "PROF",
    "WINCH",
    "LOST",
    "PWR",
    "USR1",
    "USR2",
];

pub trait Schecker {
    fn check_umask(& mut self);
    fn check_autorestart(& mut self);
    fn check_stopsignal(& mut self);
    fn trim_assign(& mut self);
}

impl Schecker for String {
    fn check_umask(& mut self) {
        *self = self.trim().to_string();
        if self.len() != 3 {
            panic!("incorrect len for umask: check it in config file");
        } else {
            for val in self.chars() {
                let val_u8: u8 = val.to_string().parse().unwrap();
                if val_u8 > 7 {
                    panic!("Incorrect value for umask: value must be between 0 and 7");
                }
            }
        }
    }
    fn check_autorestart(& mut self) {
        *self = self.trim().to_lowercase().to_string();
        if !(["true", "false", "unexpected"].contains(&self.as_str())) {
            panic!("Incorrect value for autorestart: value must be always, never or unexpected");
        }
    }
    fn check_stopsignal(& mut self) {
        *self = self.trim().to_string();
        if !(SIGNAL_NAMES.contains(&self.as_str())) {
            panic!("Incorrect value for stopsignal: value must be true, false or unexpected");
        }
    }
    fn trim_assign(& mut self) {
        *self = self.trim().to_string();
    }
}

pub trait Uchecker {
    fn u32_field_checker(&self);
}

impl Uchecker for u32 {
    fn u32_field_checker(&self) {
        if *self == 0 {
            panic!("Incorrect value: only starytime field can be 0");
        }
    }
}

pub struct Umask {
    owner: u8,
    group: u8,
    others: u8,
}

impl Umask {
    pub fn new(owner: u8, group: u8, others: u8) -> Umask {
        Umask {
            owner,
            group,
            others,
        }
    }
}

pub trait ToUmask {
    fn to_umask(& mut self) -> Umask;
}

impl ToUmask for String {
    fn to_umask(& mut self) -> Umask {
        self.check_umask();
        let mut tmp: Vec<u8> = Vec::new();
        for val in self.chars() {
            let val_u8: u8 = val.to_string().parse().unwrap();
            tmp.push(val_u8);
        }
        Umask::new(
            *tmp.get(0).unwrap(), 
            *tmp.get(1).unwrap(), 
            *tmp.get(2).unwrap(),
        )
    }
}

