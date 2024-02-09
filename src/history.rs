use std::fmt::{Display, Write};

pub trait Action {
    fn as_string(&self) -> String;
}

pub struct DummyHistory{}
impl DummyHistory{
    fn new() -> DummyHistory {
        DummyHistory{}
    }
}
impl History for DummyHistory {

    fn track(&mut self, iteration : Iteration) {}
    
}
pub trait History {
    fn track(&mut self, iteration : Iteration);
}
impl History for TrackHistory {
    fn track(&mut self, iteration : Iteration) {
        self.iterations.push(iteration)
    } 
}
pub struct TrackHistory {
    iterations : Vec<Iteration>,
}

impl TrackHistory{
    pub fn new() -> TrackHistory {
        TrackHistory{iterations : Vec::new()}
    }
}

impl Write for TrackHistory {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        todo!()
    }
}

impl Display for TrackHistory{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut res = String::new();
        for iteration in self.iterations.iter() {
            let mut iter_str = String::new();
            for actor in iteration.actors.iter(){
                if actor.actions.len() == 0 {
                    continue;
                }
                iter_str.push_str(format!("    Actor: {}\n", actor.id).as_str());
                for action in actor.actions.iter() {
                    iter_str.push_str(format!("      {}\n",action.as_string()).as_str())
                }
            }
            if iter_str.is_empty(){
                continue
            }
            res.push_str(format!("  Iteration: {}\n{}\n", iteration.nr, iter_str).as_str());
        }   

        write!(f, "{}", res)
    }
}

pub struct Iteration {
    nr : usize, 
    actors : Vec<Actor>
}
impl Iteration {
    pub fn new(nr : usize) -> Iteration {
        Iteration {nr, actors : Vec::new() }
    }

    pub fn track(&mut self, actor : Actor) {
        self.actors.push(actor)
    }
}
pub struct Actor{
    id : usize, 
    actions : Vec<Box<dyn Action>>
}

impl Actor {
    pub fn new(id : usize) -> Actor{
        Actor {id, actions : Vec::new()}
    }

    pub fn track(&mut self, action : Box<dyn Action>) {
        self.actions.push(action)
    }

}
