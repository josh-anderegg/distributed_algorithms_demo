use rand::{rngs::StdRng, Rng, SeedableRng};
use std::{cell::RefCell, rc::Rc};

pub struct Network<M> {
    links: Vec<Rc<RefCell<Link<M>>>>,
    packets: Vec<(usize, Packet<M>)>, // (ttl, packet)
    latencies: Option<Vec<Vec<usize>>>,
}
pub struct Link<M> {
    id: usize,
    in_buffer: Vec<Packet<M>>,
    out_buffer: Vec<Packet<M>>,
}

#[derive(Debug)]
pub struct Packet<M> {
    pub sender: usize,
    pub receiver: usize,
    pub content: M,
}

impl<M> Network<M> {
    pub fn new(
        asnychronous: bool,
        link_count: usize,
        seed: Option<u64>,
        max_latency: usize,
    ) -> Self {
        let links = (0..link_count)
            .map(|id| Rc::new(RefCell::new(Link::new(id))))
            .collect();

        let mut rng = match seed {
            Some(seed) => StdRng::seed_from_u64(seed),
            None => StdRng::from_entropy(),
        };

        let latencies = if asnychronous {
            Some(Self::rand_latencies(link_count, max_latency, &mut rng))
        } else {
            None
        };

        Network {
            links,
            packets: Vec::new(),
            latencies,
        }
    }

    pub fn get_link_ref(&self, id: usize) -> Rc<RefCell<Link<M>>> {
        self.links
            .get(id)
            .expect("Tried to access invalid link id")
            .clone()
    }

    pub fn get_latency(&self, message: &Packet<M>) -> usize {
        match &self.latencies {
            Some(latencies) => *latencies
                .get(message.sender)
                .expect(format!("{} not in range for valid ids", message.sender).as_str())
                .get(message.receiver)
                .expect(format!("{} not in range for valid ids", message.receiver).as_str()),
            None => 0,
        }
    }

    fn rand_latencies(link_count: usize, max_latency: usize, rng: &mut StdRng) -> Vec<Vec<usize>> {
        let mut latencies = vec![vec![0; link_count]; link_count];
        for (i, vec) in latencies.iter_mut().enumerate() {
            for (j, val) in vec.iter_mut().enumerate() {
                if i == j {
                    *val = 0
                } else {
                    *val = rng.gen_range(0..max_latency)
                }
            }
        }
        latencies
    }

    pub fn exchange_messages(&mut self) {
        self.collect_messages();
        self.deliver_messages();
    }

    fn collect_messages(&mut self) {
        let mut packets = Vec::new();
        for link in self.links.iter_mut() {
            packets.append(&mut link.borrow_mut().out_buffer.drain(..).collect());
        }
        self.packets.append(
            &mut packets
                .into_iter()
                .map(|p| (self.get_latency(&p), p))
                .collect(),
        )
    }

    fn deliver_messages(&mut self) {
        let mut remaining = Vec::new();
        for (age, message) in self.packets.drain(..) {
            if age == 0 {
                let receiver = message.receiver;
                self.links
                    .get(receiver)
                    .expect(format!("Invalid receiver id {}", receiver).as_str())
                    .borrow_mut()
                    .in_buffer
                    .push(message);
            } else {
                remaining.push((age - 1, message))
            }
        }
        self.packets = remaining
    }
}

impl<M> Link<M> {
    pub fn new(id: usize) -> Self {
        Link {
            id,
            in_buffer: Vec::new(),
            out_buffer: Vec::new(),
        }
    }

    pub fn enqueue(&mut self, receiver: usize, message: M) {
        self.out_buffer.push(Packet {
            sender: self.id,
            receiver,
            content: message,
        })
    }

    pub fn empty_buffer(&mut self) -> Vec<Packet<M>> {
        self.in_buffer.drain(..).into_iter().collect()
    }
}

mod test {

    #[test]
    fn simple_sync_system_tests() {
        use super::Network;
        use std::collections::HashSet;
        let mut system = Network::<usize>::new(false, 10, None, 0);
        system.exchange_messages();
        let contents: HashSet<usize> = system.links[0]
            .borrow_mut()
            .in_buffer
            .iter()
            .map(|p| p.content)
            .collect();
        let expected_set = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9].into_iter().collect();
        assert_eq!(contents, expected_set);
    }

    #[test]
    fn simple_exchange() {
        use super::Network;

        let mut network = Network::<usize>::new(true, 2, None, 10);
        let first = network.get_link_ref(0);
        let second = network.get_link_ref(1);
        first.borrow_mut().enqueue(1, 42);
        second.borrow_mut().enqueue(0, 69);
        let mut messages0: Vec<usize> = Vec::new();
        let mut messages1: Vec<usize> = Vec::new();
        let mut i = 0;
        while messages0 != vec![42] && messages1 != vec![69] && i < 10 {
            network.exchange_messages();
            messages0 = first
                .borrow_mut()
                .in_buffer
                .iter_mut()
                .map(|p| p.content)
                .collect();
            messages1 = second
                .borrow_mut()
                .in_buffer
                .iter_mut()
                .map(|p| p.content)
                .collect();
            i += 1;
        }
        assert_eq!(messages0, vec![69]);
        assert_eq!(messages1, vec![42]);
    }
}
