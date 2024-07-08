use std::fmt::Debug;
use std::{fs::File, io::Write};

pub mod network;
pub mod paxos;

pub trait System {
    fn new_rand(node_count: usize, server_count: usize, seed: Option<u64>) -> Self;
    fn simulate(&mut self, max_rounds: Option<usize>, log: Option<&str>);
    fn decided(&self) -> bool;
}

pub struct Logger {
    file: File,
    logging: bool,
}

impl Logger {
    pub fn new(name: Option<&str>) -> Self {
        match name {
            Some(name) => {
                let now = chrono::Local::now();
                let timestamp = now.format("%Y%m%d_%H%M%S").to_string();
                let path = format!("target/logs/{name}_{timestamp}.log");
                let file = File::create(path).expect("Path is invalid");
                Logger {
                    file,
                    logging: true,
                }
            }
            None => Logger {
                file: File::create("target/logs/dummy.log").unwrap(),
                logging: false,
            },
        }
    }

    pub fn log_round(&mut self, round: usize) {
        if !self.logging {
            return;
        }
        let log_string = format!("Iteration nr: {round}\n");
        self.file
            .write_all(log_string.as_bytes())
            .expect("Couldn't log the round")
    }

    pub fn log_actor<Actor>(&mut self, actor: &Actor)
    where
        Actor: Debug,
    {
        if !self.logging {
            return;
        }
        let log_string = format!("\t{actor:?}\n");
        self.file
            .write_all(log_string.as_bytes())
            .expect("Couldn't log the actor")
    }

    pub fn log_action<Action>(&mut self, action: &Action)
    where
        Action: Debug,
    {
        if !self.logging {
            return;
        }

        let log_string = format!("\t\t{action:?}\n");
        self.file
            .write_all(log_string.as_bytes())
            .expect("Couldn't log the action")
    }
}
