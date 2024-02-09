const MAX_LATENCY : usize= 10;
use std::sync::{Arc, Mutex};
use rand::{rngs::StdRng, Rng};

pub struct Network<M> { // M = Message type
    links : Vec<Arc<Mutex<Link<M>>>>,
    packets : Vec<(usize, Packet<M>)>, // (ttl, packet)
    pub model : CommModel, 
    latencies : Vec<Vec<usize>>
}

pub struct Link<M> {
    id : usize,
    in_buffer : Vec<Packet<M>>,
    out_buffer : Vec<Packet<M>>,
}

pub struct Packet<M> {
    pub sender : usize,
    pub receiver : usize,
    pub content : M
}

pub enum CommModel{
    Asynchronous,
    Synchronous
}

impl<M> Network<M> {

    pub fn new(model : CommModel, link_count : usize, rng: &mut StdRng) -> Self {
        let mut links = Vec::new();
        for id in 0..link_count {
            links.push(Arc::new(Mutex::new(Link::new(id))))
        }

        let latencies = match model {
            CommModel::Asynchronous => Self::asynchronous_latencies(link_count, MAX_LATENCY, rng),
            CommModel::Synchronous => Self::synchronous_latencies(link_count),
        };

        Network {links, packets : Vec::new(), model, latencies}
    }

    pub fn get_link(&self, id : usize) -> Arc<Mutex<Link<M>>> {
        self.links[id].clone()
    }

    pub fn get_latency(&self, message : &Packet<M>) -> usize {
        match self.model {
            CommModel::Asynchronous => self.latencies[message.sender][message.receiver],
            CommModel::Synchronous => 0,
        }
    }

    fn asynchronous_latencies(link_count : usize, max_latency : usize, rng : &mut StdRng) -> Vec<Vec<usize>> {
        let mut ret = Vec::new();
        for i in 0..link_count {
            let acc: Vec<usize> = vec![link_count].iter().enumerate()
            .map(|(id ,c)|  if id == i {0} else {rng.gen::<usize>() % max_latency})
            .collect();
            ret.push(acc)
        }
        ret
    }

    fn synchronous_latencies(link_count : usize) -> Vec<Vec<usize>> {
        vec![vec![0 ;link_count] ; link_count]
    }

    pub fn exchange_messages(&mut self) {
        self.collect_messages();
        self.deliver_messages();
    }

    fn collect_messages(&mut self) {
        for link in &mut self.links {
            let mut with_timestamps : Vec<(usize, Packet<M>)>= link.lock().unwrap().out_buffer.drain(..).enumerate().collect();
            self.packets.append(&mut with_timestamps)
        }
    }

    fn deliver_messages(&mut self) {
        let mut remaining = Vec::new();
        for (age, message) in self.packets.drain(..) {
            if age == 0 {
                self.links[message.receiver].lock().unwrap().in_buffer.push(message);
            } else{
                remaining.push((age -1, message))
            }
        }

        self.packets = remaining
    }

}

impl<M> Link<M> {

    pub fn new(id : usize) -> Self {
        Link {id, in_buffer: Vec::new(), out_buffer: Vec::new()}
    }

    pub fn enqueue(&mut self, receiver : usize, message : M) {
        self.out_buffer.push(Packet {sender : self.id, receiver, content : message})
    }

    pub fn empty_buffer(&mut self) -> Vec<Packet<M>> {
        self.in_buffer.drain(..).into_iter().collect()
    }
}

mod test {
    use super::*;
    use rand::{rngs::StdRng, SeedableRng};

    // System where each node sends it's id to the first node
    fn simple_sync_network(n : usize) -> Network<usize> {
        let seed = 32;
        let mut rng = StdRng::seed_from_u64(seed);
        let mut ret = Network::<usize>::new(CommModel::Synchronous, n, &mut rng);
        for link in &mut ret.links {
            let mut link = link.lock().unwrap();
            let mess = link.id;
            link.enqueue(0, mess);
            // link.send_message(0, mess)
        }      
        ret
    } 
    
    
    #[test]
    fn simple_sync_system_tests() {
        use std::collections::HashSet;
        let mut s = simple_sync_network(10);
        s.exchange_messages();
        let contents : HashSet<usize> = s.links[0].lock().unwrap()
        .in_buffer.iter()
        .map(|p| p.content)
        .collect();
        let expected_set = vec![0,1,2,3,4,5,6,7,8,9].into_iter().collect();
        assert_eq!(contents, expected_set);
    }
}