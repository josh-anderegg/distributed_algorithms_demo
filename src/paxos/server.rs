use super::*;

pub struct Server {
    id : usize, 
    link : Arc<Mutex<Link<Message>>>,
    t_max : Ticket, 
    command : Command, 
    t_store : Ticket,
    decided : bool,
}

impl Server {
    pub fn get_command(&self) -> Command {
        self.command
    }

    pub fn has_decided(&self) -> bool {
        self.decided
    }

    pub fn new(id : usize, link : Arc<Mutex<Link<Message>>>) -> Self {
        Server {id, link, t_max: 0, command: Command::Undefined, t_store: 0, decided: false}
    }

    pub fn exec(&mut self, actor : &mut Actor) {
        let inbox = self.link.lock().unwrap().empty_buffer();
        for packet in inbox {
            Self::track_action(actor, Action::Receive { sender_id: packet.sender, message: packet.content });
            match packet.content {
                Message::Ask(ticket) => {
                    
                    Self::track_action(actor,Action::Check { 
                        condition: String::from("received ticket > t_max"), 
                        values: format!("{} > {}", ticket, self.t_max), 
                        result: ticket > self.t_max
                    });
                    
                    if ticket > self.t_max{
                        self.t_max = ticket;
                        Self::track_action(actor,Action::Store { var: String::from("t_max"), value: format!("{}", ticket) });
                        let message = Message::Ok(self.t_store, self.command);
                        Self::track_action(actor,Action::Send { receiver_id: packet.sender, message });
                        self.send_message(packet.sender, message);
                    }
                },
                Message::Propose(ticket, command) => {
                    
                    Self::track_action(actor,Action::Check { 
                        condition: String::from("received ticket == t_max"), 
                        values: format!("{} == {}", ticket, self.t_max), 
                        result: ticket == self.t_max
                    });
                    
                    if self.t_max == ticket {
                        Self::track_action(actor,Action::Store { var: String::from("C"), value: format!("{:?}", command) });
                        self.command = command;

                        Self::track_action(actor,Action::Store { var: String::from("t_store"), value: format!("{}", ticket) });
                        self.t_store = ticket;

                        let message = Message::Success;
                        Self::track_action(actor, Action::Send { receiver_id: packet.sender, message });
                        self.send_message(packet.sender, Message::Success)
                    }
                },
                Message::Execute(command) => {
                    Self::track_action(actor, Action::Decide { command });
                    self.command = command;
                    self.decided = true
                },
                _ => panic!("Unexpted packet received by server")
            }
        }
    }

    fn send_message(&self, receiver : usize, message : Message) {
        self.link.lock().unwrap().enqueue(receiver, message)
    }

    fn track_action(actor : &mut Actor, action : Action) {
        actor.track(Box::new(action));
    }
}