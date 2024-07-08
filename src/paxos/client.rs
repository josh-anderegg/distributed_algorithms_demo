use super::*;
pub struct Client {
    id: usize,
    wait_duration: usize,
    link: LinkInterface,
    servers: ServerList,
    command: Command,
    cur_ticket: Ticket,
    state: usize, // 0 = ask for ticket, 1 = proposing, 2 = success
    inbox: Vec<Packet<Message>>,
}

impl Node for Client {
    fn exec(&mut self, logger: &mut Logger) {
        logger.log_actor(self);
        self.get_mail();
        let server_count = self.servers.borrow_mut().len();
        match self.state {
            0 => {
                self.inbox.clear();
                self.cur_ticket += 1;
                logger.log_action(&Action::Store(
                    String::from("t"),
                    format!("{}", self.cur_ticket),
                ));
                for server_id in self.servers.borrow_mut().iter() {
                    let message = Message::Ask(self.cur_ticket);
                    self.send_message(*server_id, message);
                    logger.log_action(&Action::Send(*server_id, message));
                }
                logger.log_action(&Action::StateChange(0, 1));
                self.state = 1;
                self.reset_wait();
            }
            1 => {
                self.inbox
                    .retain(|x| matches!(x.content, Message::Ok(_, _)));
                self.inbox
                    .iter()
                    .for_each(|m| logger.log_action(&Action::Receive(m.sender, m.content)));
                logger.log_action(&Action::Check(
                    String::from("#received ok's > #nr servers / 2"),
                    format!("{} > {}", self.inbox.len(), server_count),
                    self.inbox.len() > server_count / 2,
                ));

                if self.inbox.len() > server_count / 2 {
                    let mut max = 0;
                    for p in self.inbox.iter() {
                        match p.content {
                            Message::Ok(t_tstore, c) => {
                                if t_tstore > max && t_tstore > 0 {
                                    max = t_tstore;
                                    logger.log_action(&Action::Store(
                                        String::from("c"),
                                        format!("{:?}", c),
                                    ));
                                    self.command = c;
                                }
                            }
                            _ => (),
                        }
                    }

                    for p in self.inbox.iter() {
                        let message = Message::Propose(self.cur_ticket, self.command);
                        logger.log_action(&Action::Send(p.sender, message));
                        self.send_message(p.sender, message)
                    }
                    logger.log_action(&Action::StateChange(1, 2));
                    self.state = 2;
                    self.reset_wait();
                } else {
                    if self.wait_duration == 0 {
                        logger.log_action(&Action::StateChange(1, 0));
                        self.state = 0;
                    } else {
                        self.wait_duration -= 1;
                    }
                }
            }
            2 => {
                self.inbox.retain(|x| matches!(x.content, Message::Success));
                self.inbox
                    .iter()
                    .for_each(|m| logger.log_action(&Action::Receive(m.sender, m.content)));

                logger.log_action(&Action::Check(
                    String::from("#received successes' > #nr servers / 2"),
                    format!("{} > {}", self.inbox.len(), server_count),
                    self.inbox.len() > server_count / 2,
                ));

                if self.inbox.len() > server_count / 2 {
                    for server in self.servers.borrow_mut().iter() {
                        let message = Message::Execute(self.command);
                        logger.log_action(&Action::Send(*server, message));
                        self.send_message(*server, message)
                    }
                    logger.log_action(&Action::StateChange(2, 3));
                    self.state = 3;
                    self.reset_wait();
                } else {
                    if self.wait_duration == 0 {
                        logger.log_action(&Action::StateChange(2, 0));
                        self.state = 0;
                    } else {
                        self.wait_duration -= 1;
                    }
                }
            }
            3 => self.inbox.clear(),
            _ => panic!("Unexptected Client state"),
        }
    }

    fn get_command(&self) -> Command {
        self.command
    }

    fn has_decided(&self) -> bool {
        true
    }

    fn get_type(&self) -> usize {
        CLIENT
    }
}

impl Debug for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Client #{}", self.id)
    }
}

impl Client {
    pub fn new(id: usize, link: LinkInterface, servers: ServerList, command: Command) -> Self {
        Client {
            wait_duration: 0,
            id,
            state: 0,
            cur_ticket: 0,
            command,
            link,
            servers,
            inbox: Vec::new(),
        }
    }

    pub fn new_rand(id: usize, link: LinkInterface, servers: ServerList, rng: &mut StdRng) -> Self {
        let random_command = rng.gen::<bool>();
        Client {
            wait_duration: 0,
            id,
            state: 0,
            cur_ticket: 0,
            command: Command::Defined(random_command),
            link,
            servers,
            inbox: Vec::new(),
        }
    }

    fn get_mail(&mut self) {
        self.inbox.extend(self.link.borrow_mut().empty_buffer());
    }

    fn send_message(&self, receiver: usize, message: Message) {
        self.link.borrow_mut().enqueue(receiver, message)
    }

    fn reset_wait(&mut self) {
        self.wait_duration = WAIT_DURATION;
    }
}
