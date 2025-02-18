# Taskmaster
## Overview

Taskmaster is a job control daemon similar to Supervisor. It starts, monitors, and manages processes based on configurations defined in a file. Implemented in Rust, Taskmaster ensures process uptime by restarting failed processes and provides a control shell for user interactions.
## Features

-   Start and manage jobs as child processes.
-   Monitor process states accurately.
-   Reload configurations without stopping the main program.
-   Logging system for events.
-   Control shell for user commands (status, start/stop/restart programs, reload configuration, stop main program).

## Prerequisites

-   A Unix-like operating system.
-   Rust installed.

## Installation
```
git clone https://github.com/jbettini/Taskmaster.git
cd Taskmaster
```

## Usage

Create a config file inside ./confs

### Example Configuration File:
```
programs:
  nginx:
    cmd: "/usr/local/bin/nginx -c /etc/nginx/test.conf"
    numprocs: 1
    umask: 022
    workingdir: /tmp
    autostart: true
    autorestart: unexpected
    exitcodes:
      - 0
      - 2
    startretries: 3
    starttime: 5
    stopsignal: TERM
    stoptime: 10
    stdout: /tmp/nginx.stdout
    stderr: /tmp/nginx.stderr
    env:
      STARTED_BY: taskmaster
      ANSWER: 42
```
### Run Client
```
cargo run --bin taskmasterctl
```

### Run Server
```
cargo run --bin taskmasterd
```

## Control Shell Commands

-   status: See the status of all programs.
-   start [program]: Start a specific program.
-   stop [program]: Stop a specific program.
-   restart [program]: Restart a specific program.
-   reload: Reload the configuration file.
-   exit: Stop the main program.

## Bonus Features

    Client/server architecture: Separate daemon and control program communicating over UNIX or TCP sockets.
