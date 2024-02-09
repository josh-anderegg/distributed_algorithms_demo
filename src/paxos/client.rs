use super::*;
pub struct Client {
    id : usize,
    wait_duration : usize,
    link : Arc<Mutex<Link<Message>>>,
    servers : Arc<Mutex<Vec<usize>>>,
    command : Command,
    cur_ticket: Ticket,
    state : usize, // 0 = ask for ticket, 1 = proposing, 2 = success
    inbox : Vec<Packet<Message>>,
}

impl crate::Node for Client {
    fn exec(&mut self, actor : &mut Actor) {
        self.get_mail();
        let server_count = self.servers.lock().unwrap().len();
        
        match self.state {
            0 => {
                self.inbox.clear();
                self.cur_ticket += 1;
                Self::track_action(actor, Action::Store { var: String::from("t"), value: format!("{}", self.cur_ticket) });
                for server_id in self.servers.lock().unwrap().iter(){
                    let message = Message::Ask(self.cur_ticket);
                    self.send_message(*server_id, message);
                    
                    Self::track_action(actor, Action::Send { receiver_id: *server_id, message});
                }
                Self::track_action(actor, Action::StateChange { from: 0, to: 1 });
                self.state = 1;
                self.reset_wait();
            },
            1 => {
                
                self.inbox.retain(|x| matches!(x.content, Message::Ok(_,_)));
                self.inbox.iter().for_each(|m| Self::track_action(actor, Action::Receive { sender_id: m.sender, message: m.content }));

                Self::track_action(actor, Action::Check { 
                    condition: String::from("#received ok's > #nr servers / 2"), 
                    values: format!("{} > {}", self.inbox.len(), server_count), 
                    result: self.inbox.len() > server_count/2 
                });

                if self.inbox.len() > server_count/2 {
                    let mut max = 0;
                    for p in self.inbox.iter() {
                        match p.content {
                            Message::Ok(t_tstore, c) => {
                                if t_tstore > max && t_tstore > 0 {
                                    max = t_tstore;
                                    Self::track_action(actor, Action::Store { var: String::from("c"), value: format!("{:?}", c) });
                                    self.command = c;
                                }
                            },
                            _ => (),
                        }
                    }

                    for p in self.inbox.iter(){
                        let message = Message::Propose(self.cur_ticket,self.command);
                        Self::track_action(actor, Action::Send { receiver_id: p.sender, message: message });
                        self.send_message(p.sender, message)
                    }
                    
                    Self::track_action(actor, Action::StateChange { from: 1, to: 2 });
                    self.state = 2;
                    self.reset_wait();
                } else{
                    if self.wait_duration == 0 {
                        Self::track_action(actor, Action::StateChange { from: 1, to: 0 });
                        self.state = 0;
                    } else {
                        self.wait_duration -= 1;
                    }
                }
            },
            2 => {
                self.inbox.retain(|x| matches!(x.content, Message::Success));
                self.inbox.iter().for_each(|m| Self::track_action(actor, Action::Receive { sender_id: m.sender, message: m.content }));

                Self::track_action(actor, Action::Check { 
                    condition: String::from("#received successes' > #nr servers / 2"), 
                    values: format!("{} > {}", self.inbox.len(), server_count), 
                    result: self.inbox.len() > server_count/2 
                });

                if self.inbox.len() > server_count/2 {
                    for server in self.servers.lock().unwrap().iter() {
                        let message = Message::Execute(self.command);
                        Self::track_action(actor, Action::Send { receiver_id: *server, message: message });
                        self.send_message(*server, message)
                    }
                    Self::track_action(actor, Action::StateChange { from: 2, to: 3 });
                    self.state = 3;
                    self.reset_wait();
                } else{
                    if self.wait_duration == 0 {
                        Self::track_action(actor, Action::StateChange { from: 2, to: 0 });
                        self.state = 0;
                    } else {
                        self.wait_duration -= 1;
                    }
                }
            },
            3 => self.inbox.clear(),
            _ => panic!("Unexptected Client state") 
        }
    }

    fn send_message<M: Clone + Copy>(&self, receiver : usize, literal : M) {
        self.link.lock().unwrap().enqueue(receiver, literal);
    }

    fn global_data(&self) {
        todo!()
    }
}

impl Client{
    pub fn get_command(&self) -> Command {
        self.command
    }
    
    pub fn new(id : usize, link : Arc<Mutex<Link<Message>>>, servers : Arc<Mutex<Vec<usize>>>, command : Command) -> Self {
        Client {wait_duration : 0, id, state : 0, cur_ticket : 0, command, link, servers, inbox : Vec::new()}
    }

    pub fn new_rand(id : usize, link : Arc<Mutex<Link<Message>>>, servers : Arc<Mutex<Vec<usize>>>, rng : &mut StdRng) -> Self {
        let random_command = rng.gen::<bool>();
        Client {wait_duration: 0, id, state : 0, cur_ticket : 0, command : Command::Defined(random_command), link, servers, inbox : Vec::new()}
    }

    fn get_mail(&mut self) {
        self.inbox.extend(self.link.lock().unwrap().empty_buffer());     
    }


    pub fn exec(&mut self, actor : &mut Actor)  {   
        self.get_mail();
        let server_count = self.servers.lock().unwrap().len();
        
        match self.state {
            0 => {
                self.inbox.clear();
                self.cur_ticket += 1;
                Self::track_action(actor, Action::Store { var: String::from("t"), value: format!("{}", self.cur_ticket) });
                for server_id in self.servers.lock().unwrap().iter() {
                    let message = Message::Ask(self.cur_ticket);
                    self.send_message(*server_id, message);
                    
                    Self::track_action(actor, Action::Send { receiver_id: *server_id, message});
                }
                Self::track_action(actor, Action::StateChange { from: 0, to: 1 });
                self.state = 1;
                self.reset_wait();
            },
            1 => {
                
                self.inbox.retain(|x| matches!(x.content, Message::Ok(_,_)));
                self.inbox.iter().for_each(|m| Self::track_action(actor, Action::Receive { sender_id: m.sender, message: m.content }));

                Self::track_action(actor, Action::Check { 
                    condition: String::from("#received ok's > #nr servers / 2"), 
                    values: format!("{} > {}", self.inbox.len(), server_count), 
                    result: self.inbox.len() > server_count/2 
                });

                if self.inbox.len() > server_count/2 {
                    let mut max = 0;
                    for p in self.inbox.iter() {
                        match p.content {
                            Message::Ok(t_tstore, c) => {
                                if t_tstore > max && t_tstore > 0 {
                                    max = t_tstore;
                                    Self::track_action(actor, Action::Store { var: String::from("c"), value: format!("{:?}", c) });
                                    self.command = c;
                                }
                            },
                            _ => (),
                        }
                    }

                    for p in self.inbox.iter(){
                        let message = Message::Propose(self.cur_ticket,self.command);
                        Self::track_action(actor, Action::Send { receiver_id: p.sender, message: message });
                        self.send_message(p.sender, message)
                    }
                    Self::track_action(actor, Action::StateChange { from: 1, to: 2 });
                    self.state = 2;
                    self.reset_wait();
                } else{
                    if self.wait_duration == 0 {
                        Self::track_action(actor, Action::StateChange { from: 1, to: 0 });
                        self.state = 0;
                    } else {
                        self.wait_duration -= 1;
                    }
                }
            },
            2 => {
                self.inbox.retain(|x| matches!(x.content, Message::Success));
                self.inbox.iter().for_each(|m| Self::track_action(actor, Action::Receive { sender_id: m.sender, message: m.content }));

                Self::track_action(actor, Action::Check { 
                    condition: String::from("#received successes' > #nr servers / 2"), 
                    values: format!("{} > {}", self.inbox.len(), server_count), 
                    result: self.inbox.len() > server_count/2 
                });

                if self.inbox.len() > server_count/2 {
                    for server in self.servers.lock().unwrap().iter() {
                        let message = Message::Execute(self.command);
                        Self::track_action(actor, Action::Send { receiver_id: *server, message: message });
                        self.send_message(*server, message)
                    }
                    Self::track_action(actor, Action::StateChange { from: 2, to: 3 });
                    self.state = 3;
                    self.reset_wait();
                } else{
                    if self.wait_duration == 0 {
                        Self::track_action(actor, Action::StateChange { from: 2, to: 0 });
                        self.state = 0;
                    } else {
                        self.wait_duration -= 1;
                    }
                }
            },
            3 => self.inbox.clear(),
            _ => panic!("Unexptected Client state") 
        }
    }   
    

    fn send_message(&self, receiver : usize, message : Message) {
        self.link.lock().unwrap().enqueue(receiver, message)
    }

    fn reset_wait(&mut self) {
        self.wait_duration = WAIT_DURATION;
    }

    fn track_action(actor : &mut Actor, action : Action) {
        actor.track(Box::new(action));
    }
}
