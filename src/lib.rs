#![allow(dead_code, unused_variables)]
mod communication;
// mod paxos;
mod history;
use history::{Actor, TrackHistory};

trait Node {
    fn exec(&mut self, actor : &mut Actor);
    fn send_message<M>(&self, receiver : usize, message : Box<M>);
    fn global_data(&self );
}

trait System {
    fn simulate(history : &mut TrackHistory);
    fn global_data();
}


// #[cfg(test)]
// mod tests {

//     use rand::{rngs::StdRng, SeedableRng};
//     use self::history::TrackHistory;

//     use super::*;
//     use paxos::System;
    
//     #[test]
//     fn rng_is_deterministic() {
//         let seed = 64;
//         let mut rng1 = StdRng::seed_from_u64(seed);
//         let system1 = System::new_rand(20, 2, &mut rng1);
//         let mut rng2 = StdRng::seed_from_u64(seed);
//         let system2 = System::new_rand(20, 2, &mut rng2);
//         assert_eq!(system1.client_commands(), system2.client_commands());
//     }

//     #[test]
//     fn paxos_3(){
//         let seed = 32;
//         let mut rng = StdRng::seed_from_u64(seed);
//         let mut system = System::new_rand(3, 1, &mut rng);
//         let mut history = TrackHistory::new();
//         system.simulate(None, &mut history);
//     }
//     #[test]
//     fn terminates(){
//         let seed = 420;
//         let mut rng = StdRng::seed_from_u64(seed);
//         let mut system = System::new_rand(3, 1, &mut rng);
//         let mut history = TrackHistory::new();
//         system.simulate(None, &mut history);
//     }

//     #[test]
//     fn agrees(){
//         let seed = 420;
//         let mut rng = StdRng::seed_from_u64(seed);
//         let mut system = System::new_rand(3, 1, &mut rng);
//         let mut history = TrackHistory::new();
//         system.simulate(None, &mut history);
//         assert!(system.servers_agree() != None)
//     }

//     #[test]
//     fn valid_history(){
//         let seed = 420;
//         let mut rng = StdRng::seed_from_u64(seed);
//         let mut system = System::new_rand(3, 1, &mut rng);
//         let mut history = TrackHistory::new();
//         system.simulate(None, &mut history);
//         println!("{history}")
//     }
// }
