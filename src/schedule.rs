use petgraph::graphmap::DiGraphMap;
use petgraph::visit::Bfs;
use std::collections::HashMap;

use super::ecs::{System, World};

// pub type ScheduleLabel = u32;
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ScheduleLabel {
    Start,    // Runs once on startup.
    First,    // Runs before any main update code.
    Update,   // Main app logic.
    Last,     // Runs after main update code.
    Clear,    // Clear state for next loop.
    Shutdown, // Run before exiting.
}

#[derive(Clone)]
pub struct Scheduler {
    schedule: HashMap<ScheduleLabel, Vec<System>>,
    pub started: bool,
    pub shutdown: bool,
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            schedule: HashMap::new(),
            started: false,
            shutdown: false,
        }
    }

    pub fn build_graph(&mut self) -> DiGraphMap<ScheduleLabel, ()> {
        println!("shutdown: {}", self.shutdown);
        let dag;
        if !self.started {
            dag = DiGraphMap::<ScheduleLabel, ()>::from_edges(&[
                (ScheduleLabel::Start, ScheduleLabel::First),
                (ScheduleLabel::First, ScheduleLabel::Update),
                (ScheduleLabel::Update, ScheduleLabel::Last),
                (ScheduleLabel::Last, ScheduleLabel::Clear),
            ])
        } else if self.shutdown {
            dag = DiGraphMap::<ScheduleLabel, ()>::from_edges(&[
                (ScheduleLabel::First, ScheduleLabel::Update),
                (ScheduleLabel::Update, ScheduleLabel::Last),
                (ScheduleLabel::Last, ScheduleLabel::Clear),
                (ScheduleLabel::Clear, ScheduleLabel::Shutdown),
            ])
        } else {
            dag = DiGraphMap::<ScheduleLabel, ()>::from_edges(&[
                (ScheduleLabel::First, ScheduleLabel::Update),
                (ScheduleLabel::Update, ScheduleLabel::Last),
                (ScheduleLabel::Last, ScheduleLabel::Clear),
            ])
        }
        return dag;
    }

    pub fn execute(&mut self, world: &mut World) -> bool {
        let dag = self.build_graph();
        let start = dag.nodes().nth(0).unwrap();
        let mut bfs = Bfs::new(&dag, start);
        while let Some(label) = bfs.next(&dag) {
            if let Some(list) = self.schedule.get_mut(&label) {
                println!("Stage: {:?}", label);
                for system in list.iter() {
                    system(world).expect("systems don't support returning errors yet.");
                }
            }
        }
        self.shutdown
    }

    pub fn add_system(&mut self, label: ScheduleLabel, system: System) {
        let list = self.schedule.entry(label).or_insert(Vec::new());
        list.push(system);
    }
}
