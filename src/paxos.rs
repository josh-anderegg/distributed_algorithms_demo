use crate::communication::{CommModel, Link, Network, Packet};
use crate::history::{self, Actor, History, Iteration, TrackHistory};
use std::{fmt::Debug, sync::{Arc, Mutex}, usize::MAX};
use rand::{self, rngs::StdRng, Rng};
use server::Server;
use client::Client;

const WAIT_DURATION : usize = 50;

mod server;
mod client;

type Ticket = usize;

#[derive(Debug, Clone, Copy)]
pub enum Message {
    Ask(Ticket),
    Ok(Ticket, Command),
    Propose(Ticket, Command),
    Success,
    Execute(Command)
}

#[derive(Clone, Copy, PartialEq)]
pub enum Command {
    Defined(bool),
    Undefined
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

pub enum Node {
    Server(Server),
    Client(Client)
}

enum Action {
    Store{var : String, value : String},
    Send{receiver_id : usize, message: Message},
    StateChange{from : usize, to : usize},
    Receive{sender_id : usize, message : Message},
    Check{condition : String, values : String, result : bool},
    Decide{command : Command}
}

impl history::Action for Action{
    fn as_string(&self) -> String {
        match self {
            Action::Store { var, value } => format!("store {var} = {value}"),
            Action::Send { receiver_id, message } => format!("send {message:?} to {receiver_id}"),
            Action::StateChange { from, to } => format!("change state from {from} to {to}"),
            Action::Receive { sender_id, message } => format!("received {message:?} from {sender_id}"),
            Action::Check { condition, values, result } => format!("check {condition}: {values} => {result}"),
            Action::Decide { command } => format!("decides for {command:?}"),
        }
    }
}


pub struct System {
    nodes : Vec<Node>,
    network : Network<Message>,
    servers : Arc<Mutex<Vec<usize>>>
}

impl crate::System for System {
    
    fn new_rand(node_count : usize, server_count : usize, rng : &mut StdRng) -> Self {
        let network = Network::new(CommModel::Asynchronous, node_count, rng);
        let mut id = 0;
        let mut nodes = Vec::new();
        let servers = Arc::new(Mutex::new(Vec::new()));

        while id < server_count {
            nodes.push(Node::Server(Server::new(id, network.get_link(id))));
            servers.lock().unwrap().push(id);
            id += 1
        }

        while id < node_count {
            nodes.push(Node::Client(Client::new_rand(id, network.get_link(id), servers.clone(), rng)));
            id += 1
        }

        System {nodes, network, servers}
    }

    fn simulate(&mut self, rounds : Option<usize>, history : &mut TrackHistory) {
        let rounds = match rounds {
            Some(nr) => nr,
            None => MAX,
        };

        let mut iteration_nr = 0 ;
        while !self.decided() && (rounds == MAX || iteration_nr < rounds){
            let mut iteration = Iteration::new(iteration_nr);
            
            self.network.exchange_messages();
            
            for (id, node) in self.nodes.iter_mut().enumerate(){
                let mut actor = Actor::new(id);
                node.exec(&mut actor);
                iteration.track(actor)
            }

            history.track(iteration);
            iteration_nr += 1;
        }

    }

    fn decided(&self) -> bool {
        for node in self.nodes.iter() {
            match node {
                Node::Server(server) => if !server.has_decided() {
                    return false
                } ,
                _ => (),
            }
        }
        true
    }
    

}

impl System {

    pub fn client_commands(&self) -> Vec<Command> {
        let mut ret = Vec::new();
        for node in self.nodes.iter() {
            match node {
                Node::Client(client) => ret.push(client.get_command()),
                _ => (),
            }
        }

        return ret
    }

    pub fn servers_agree(&self) -> Option<Command> {
        let mut any_command = None;
        for node in self.nodes.iter(){
            match node {
                Node::Server(server) => {
                    any_command = Some(server.get_command());
                    break
                },
                _ => (),
            }
        }
        
        for node in self.nodes.iter(){
            match node {
                Node::Server(server) => {
                    if server.get_command() != any_command.unwrap() {
                        return None
                    }
                },
                _ => (),
            }
        }
        any_command
    }
}

impl Node {
    pub fn exec(&mut self, actor : &mut Actor) {
        match self  {
            Node::Client(client) => client.exec(actor),
            Node::Server(server) => server.exec(actor)
        };
    }

}


#[cfg(test)]
mod test {
    use super::*;
    use crate::*;
    use rand::rngs::StdRng;
    use rand::SeedableRng;
    use crate::System;

    #[test]
    fn paxos_rng_is_deterministic() {
        let seed = 64;
        let mut rng1 = StdRng::seed_from_u64(seed);
        let system1:paxos::System = System::new_rand(20, 2, &mut rng1);
        let mut rng2 = StdRng::seed_from_u64(seed);
        let system2:paxos::System = System::new_rand(20, 2, &mut rng2);
        assert_eq!(system1.client_commands(), system2.client_commands());
    }

    #[test]
    fn paxos_3(){
        let seed = 32;
        let mut rng = StdRng::seed_from_u64(seed);
        let mut system:paxos::System = System::new_rand(3, 1, &mut rng);
        let mut history = TrackHistory::new();
        system.simulate(None, &mut history);
    }

    #[test]
    fn paxos_terminates(){
        let seed = 420;
        let mut rng = StdRng::seed_from_u64(seed);
        let mut system:paxos::System = System::new_rand(3, 1, &mut rng);
        let mut history = TrackHistory::new();
        system.simulate(None, &mut history);
    }

    #[test]
    fn paxos_agrees(){
        let seed = 420;
        let mut rng = StdRng::seed_from_u64(seed);
        let mut system:paxos::System = System::new_rand(3, 1, &mut rng);
        let mut history = TrackHistory::new();
        system.simulate(None, &mut history);
        assert!(system.servers_agree() != None)
    }

    #[test]
    fn paxos_valid_history(){
        let seed = 420;
        let mut rng = StdRng::seed_from_u64(seed);
        let mut system:paxos::System = System::new_rand(3, 1, &mut rng);
        let mut history = TrackHistory::new();
        system.simulate(None, &mut history);
    }
}