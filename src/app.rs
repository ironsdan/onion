use hecs::World;
use std::error::Error;

pub type System = Box<dyn Fn(&mut World) -> Result<(), Box<dyn Error>>>;

pub struct App {
    pub world: World,
    systems: Vec<System>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            world: World::new(),
            systems: Vec::new(),
        }
    }
}

impl App {
    pub fn new() -> Self {
        App::default()
    }

    pub fn add_system(&mut self, system: System) -> &mut Self {
        self.systems.push(system);
        self
    }

    pub fn run(&mut self) {
        loop {
            for system in self.systems.iter() {
                if let Err(e) = system(&mut self.world) {
                    panic!("system errors aren't supported yet: {e:?}");
                }
            }
        }
    }
}
