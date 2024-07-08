use super::*;
pub struct Server {
    id: usize,
    link: LinkInterface,
    t_max: Ticket,
    command: Command,
    t_store: Ticket,
    decided: bool,
}

impl Node for Server {
    fn exec(&mut self, logger: &mut Logger) {
        logger.log_actor(self);
        let inbox = self.link.borrow_mut().empty_buffer();
        for packet in inbox {
            logger.log_action(&Action::Receive(packet.sender, packet.content));
            match packet.content {
                Message::Ask(ticket) => {
                    logger.log_action(&Action::Check(
                        String::from("received ticket > t_max"),
                        format!("{} > {}", ticket, self.t_max),
                        ticket > self.t_max,
                    ));

                    if ticket > self.t_max {
                        self.t_max = ticket;
                        logger.log_action(&Action::Store(
                            String::from("t_max"),
                            format!("{}", ticket),
                        ));
                        let message = Message::Ok(self.t_store, self.command);
                        logger.log_action(&Action::Send(packet.sender, message));
                        self.send_message(packet.sender, message);
                    }
                }
                Message::Propose(ticket, command) => {
                    logger.log_action(&Action::Check(
                        String::from("received ticket == t_max"),
                        format!("{} == {}", ticket, self.t_max),
                        ticket == self.t_max,
                    ));

                    if self.t_max == ticket {
                        logger.log_action(&Action::Store(
                            String::from("C"),
                            format!("{:?}", command),
                        ));
                        self.command = command;

                        logger.log_action(&Action::Store(
                            String::from("t_store"),
                            format!("{}", ticket),
                        ));
                        self.t_store = ticket;

                        let message = Message::Success;
                        logger.log_action(&Action::Send(packet.sender, message));
                        self.send_message(packet.sender, Message::Success)
                    }
                }
                Message::Execute(command) => {
                    logger.log_action(&Action::Decide(command));
                    self.command = command;
                    self.decided = true
                }
                _ => panic!("Unexpted packet received by server"),
            }
        }
    }

    fn get_command(&self) -> Command {
        self.command
    }

    fn has_decided(&self) -> bool {
        self.decided
    }

    fn get_type(&self) -> usize {
        SERVER
    }
}
impl Debug for Server {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Server #{}", self.id)
    }
}
impl Server {
    pub fn new(id: usize, link: Rc<RefCell<Link<Message>>>) -> Self {
        Server {
            id,
            link,
            t_max: 0,
            command: Command::Undefined,
            t_store: 0,
            decided: false,
        }
    }

    fn send_message(&self, receiver: usize, message: Message) {
        self.link.borrow_mut().enqueue(receiver, message)
    }
}
