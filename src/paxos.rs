pub mod client;
pub mod server;

use super::Logger;
use crate::network::{Link, Network, Packet};
use client::Client;
use rand::SeedableRng;
use rand::{self, rngs::StdRng, Rng};
use server::Server;
use std::{cell::RefCell, fmt::Debug, rc::Rc, usize::MAX};

const WAIT_DURATION: usize = 50;
const SERVER: usize = 0;
const CLIENT: usize = 1;
type Ticket = usize;
type ServerList = Rc<RefCell<Vec<usize>>>;
type LinkInterface = Rc<RefCell<Link<Message>>>;

#[derive(Debug, Clone, Copy)]
pub enum Message {
    Ask(Ticket),
    Ok(Ticket, Command),
    Propose(Ticket, Command),
    Success,
    Execute(Command),
}

#[derive(Clone, Copy, PartialEq)]
pub enum Command {
    Defined(bool),
    Undefined,
}

impl Debug for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Defined(true) => write!(f, "1"),
            Self::Defined(false) => write!(f, "0"),
            Self::Undefined => write!(f, "⊥"),
        }
    }
}

trait Node {
    fn exec(&mut self, logger: &mut Logger);
    fn get_command(&self) -> Command;
    fn has_decided(&self) -> bool;
    fn get_type(&self) -> usize;
}
enum Action {
    Store(String, String),
    Send(usize, Message),
    StateChange(usize, usize),
    Receive(usize, Message),
    Check(String, String, bool),
    Decide(Command),
}

impl Debug for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Store(var, value) => write!(f, "store {var} = {value}"),
            Self::Send(receiver_id, message) => write!(f, "send {message:?} to {receiver_id}"),
            Self::StateChange(from, to) => write!(f, "change state from {from} to {to}"),
            Self::Receive(sender_id, message) => {
                write!(f, "received {message:?} from {sender_id}")
            }
            Self::Check(condition, values, result) => {
                write!(f, "check {condition}: {values} => {result}")
            }
            Self::Decide(command) => write!(f, "decides for {command:?}"),
        }
    }
}

pub struct System {
    nodes: Vec<Box<dyn Node>>,
    network: Network<Message>,
}

impl crate::System for System {
    fn new_rand(node_count: usize, server_count: usize, seed: Option<u64>) -> Self {
        let mut rng = match seed {
            Some(seed) => StdRng::seed_from_u64(seed),
            None => StdRng::from_entropy(),
        };

        let network = Network::new(true, node_count, seed, 10);
        let mut id = 0;
        let mut nodes: Vec<Box<dyn Node>> = Vec::new();
        let servers = Rc::new(RefCell::new(Vec::with_capacity(server_count)));

        while id < server_count {
            nodes.push(Box::new(Server::new(id, network.get_link_ref(id))));
            servers.borrow_mut().push(id);
            id += 1
        }

        while id < node_count {
            nodes.push(Box::new(Client::new_rand(
                id,
                network.get_link_ref(id),
                servers.clone(),
                &mut rng,
            )));
            id += 1
        }

        System { nodes, network }
    }

    fn simulate(&mut self, max_rounds: Option<usize>, log: Option<&str>) {
        let max_rounds = match max_rounds {
            Some(nr) => nr,
            None => MAX,
        };

        let mut logger = Logger::new(log);

        let mut cur_round = 0;
        while !self.decided() && cur_round < max_rounds {
            logger.log_round(cur_round);
            self.network.exchange_messages();
            for (_, node) in self.nodes.iter_mut().enumerate() {
                node.exec(&mut logger);
            }

            cur_round += 1;
        }
    }

    fn decided(&self) -> bool {
        for node in self.nodes.iter() {
            if !node.has_decided() {
                return false;
            }
        }
        true
    }
}

impl System {
    pub fn client_commands(&self) -> Vec<Command> {
        let mut ret = Vec::new();
        for node in self.nodes.iter() {
            match node.get_type() {
                CLIENT => ret.push(node.get_command()),
                _ => (),
            }
        }

        return ret;
    }

    pub fn servers_agree(&self) -> Option<Command> {
        let mut any_command = None;
        for node in self.nodes.iter() {
            match node.get_type() {
                SERVER => {
                    any_command = Some(node.get_command());
                    break;
                }
                _ => (),
            }
        }

        for node in self.nodes.iter() {
            match node.get_type() {
                SERVER => {
                    if node.get_command() != any_command.unwrap() {
                        return None;
                    }
                }
                _ => (),
            }
        }
        any_command
    }
}

#[cfg(test)]
mod test {
    use crate::System;
    use crate::*;

    #[test]
    fn paxos_rng_is_deterministic() {
        let seed = 42;
        let system1: paxos::System = System::new_rand(20, 2, Some(seed));
        let system2: paxos::System = System::new_rand(20, 2, Some(seed));
        assert_eq!(system1.client_commands(), system2.client_commands());
    }

    #[test]
    fn paxos_3() {
        let mut system: paxos::System = System::new_rand(3, 1, None);
        system.simulate(None, Some("paxos_3"));
    }

    #[test]
    fn paxos_terminates() {
        let seed = 420;
        let mut system: paxos::System = System::new_rand(3, 1, Some(seed));
        system.simulate(None, Some("paxos_terminates"));
    }

    #[test]
    fn paxos_agrees() {
        let mut system: paxos::System = System::new_rand(3, 1, None);
        system.simulate(None, Some("paxos_agrees"));
        assert!(system.servers_agree() != None)
    }

    #[test]
    fn paxos_valid_history() {
        let seed = 420;
        let mut system: paxos::System = System::new_rand(3, 1, Some(seed));
        system.simulate(None, Some("paxos_valid"));
    }
}
