use crate::communication::{CommModel, Link, Network, Packet};
use crate::history::{self, Actor, History, Iteration};
use std::{fmt::Debug, sync::{Arc, Mutex}, usize::MAX};
use rand::{self, rngs::StdRng, Rng};
use server::Server;
use client::Client;
use crate::Node;

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

// pub enum Node {
//     Server(Server),
//     Client(Client)
// }

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
    nodes : Vec<Box<dyn Node>>,
    network : Network<Message>,
    servers : Arc<Mutex<Vec<usize>>>
}

impl System {

    pub fn new_rand(node_count : usize, server_count : usize, rng : &mut StdRng) -> Self {
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

    pub fn simulate(&mut self, rounds : Option<usize>, history : &mut dyn History) {
        let rounds = match rounds {
            Some(nr) => nr,
            None => MAX,
        };

        let mut iteration_nr = 0 ;
        while !self.servers_decided() && (rounds == MAX || iteration_nr < rounds){
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

    pub fn servers_decided(&self) -> bool {
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

}


// impl Node {
//     pub fn exec(&mut self, actor : &mut Actor) {
//         match self  {
//             Node::Client(client) => client.exec(actor),
//             Node::Server(server) => server.exec(actor)
//         };
//     }

// }

