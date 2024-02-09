#![allow(dead_code, unused_variables)]
mod communication;
mod paxos;
mod history;
use history::TrackHistory;
use rand::rngs::StdRng;

trait System {
    fn new_rand(node_count : usize, server_count : usize, rng : &mut StdRng) -> Self;
    fn simulate(&mut self, rounds : Option<usize>, history : &mut TrackHistory);
    fn decided(&self) -> bool;
}
