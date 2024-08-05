use hecs::Entity;
use petgraph::graphmap::DiGraphMap;
use petgraph::visit::Bfs;
use std::any::TypeId;
use std::collections::hash_map::Entry::Vacant;
use std::collections::HashMap;
use std::fmt::Debug;
use thiserror::Error;

// An ECS system.
pub type System = fn(&mut World) -> anyhow::Result<()>;

// A Worldly unique instance of a type.
pub trait Resource: Send + Sync + 'static {}

// Something that "happens", can be observed and trigger.
// Useful for inter-system communication.
pub trait Event: Resource + Default + Debug {}

// A list of similar events.
#[derive(Default)]
pub struct EventBuffer<E: Event> {
    pub events: Vec<E>,
}

impl<E: Event> Resource for EventBuffer<E> {}

#[derive(Error, Debug)]
pub enum ECSError {
    #[error("resource not found")]
    ResourceNotFound,
}

// An extended hecs::World. Adds resources, events and TODO: commands.
pub struct World {
    pub inner: hecs::World,
    pub scheduler: Scheduler,
    resource_map: HashMap<TypeId, Entity>,
    event_updates: HashMap<TypeId, fn(&mut World)>,
    events_enabled: bool,
    exit: bool,
}

impl Default for World {
    fn default() -> Self {
        Self {
            inner: hecs::World::new(),
            scheduler: Scheduler::new(),
            resource_map: HashMap::new(),
            event_updates: HashMap::new(),
            events_enabled: false,
            exit: false,
        }
    }
}

impl World {
    pub fn new() -> Self {
        World::default()
    }

    // Add a system to the specified schedule step.
    pub fn add_label_system(&mut self, label: ScheduleLabel, system: System) -> &mut Self {
        self.scheduler.add_system(label, system);
        self
    }

    // Add a system to the default schedule step.
    pub fn add_system(&mut self, system: System) -> &mut Self {
        self.scheduler.add_system(ScheduleLabel::Update, system);
        self
    }

    // Insert a new unique resource. Resources are Worldly unique so inserting
    // a resource that already exists will overwrite it.
    pub fn insert_resource<R: Resource>(&mut self, resource: R) -> &mut Self {
        // Can't just let the HashMap::insert replace the entity because then it would
        // live for ever and never be cleaned up.
        if let Some(_) = self.resource_map.get(&TypeId::of::<R>()) {
            self.remove_resource::<R>();
        }
        let e = self.inner.spawn((resource,));
        self.resource_map.insert(TypeId::of::<R>(), e);
        self
    }

    // Return a unique reference to a resource or an error if it didn't exist.
    pub fn get_resource_mut<R: Resource>(&mut self) -> Option<hecs::RefMut<'_, R>> {
        let e = self.resource_map.get(&TypeId::of::<R>())?;
        // If the entity doesn't exist and resource_map had the value still that's a bug.
        Some(
            self.inner
                .get::<&mut R>(*e)
                .expect("resource_map entry existed for a nonexistent entity"),
        )
    }

    // Return a shared reference to a resource or an error if it didn't exist.
    pub fn get_resource<R: Resource>(&mut self) -> Option<hecs::Ref<'_, R>> {
        let e = self.resource_map.get(&TypeId::of::<R>())?;
        // If the entity doesn't exist and resource_map had the value still that's a bug.
        Some(
            self.inner
                .get::<&R>(*e)
                .expect("resource_map entry existed for a nonexistent entity"),
        )
    }

    // Delete a resource or an error if it didn't exist. Noop if the resource doesn't exist.
    pub fn remove_resource<R: Resource>(&mut self) -> bool {
        if let Some(e) = self.resource_map.get(&TypeId::of::<R>()) {
            self.inner
                .get::<&R>(*e)
                .expect("resource_map entry existed for a nonexistent entity");
            self.inner
                .despawn(*e)
                .expect("resource_map entry existed for a nonexistent entity");
            return true;
        }
        return false;
    }

    // Must be called before using an event in trigger. Running register_event on an
    // already registered event is a noop.
    pub fn register_event<E: Event>(&mut self) -> &mut Self {
        if !self.events_enabled {
            self.add_label_system(ScheduleLabel::Clear, clear_events);
        }
        self.events_enabled = true;
        if let Vacant(k) = self.resource_map.entry(TypeId::of::<EventBuffer<E>>()) {
            let e = self.inner.spawn((EventBuffer::<E>::default(),));
            k.insert(e);
            self.event_updates
                .insert(TypeId::of::<EventBuffer<E>>(), |world| {
                    world
                        .get_resource_mut::<EventBuffer<E>>()
                        .unwrap()
                        .events
                        .clear();
                });
        }

        self
    }

    // Clean up an event when it will no longer be used. Calling this on a unregistered
    // event will return an error. Noop if the event didn't exist.
    pub fn deregister_event<E: Event>(&mut self) {
        if self.remove_resource::<EventBuffer<E>>() {
            self.event_updates.remove(&TypeId::of::<EventBuffer<E>>());
        }
    }

    // Send an event to the event queue for observers to view react to.
    pub fn trigger<E: Event>(&mut self, event: E) -> anyhow::Result<()> {
        if let Some(mut e) = self.get_resource_mut::<EventBuffer<E>>() {
            e.events.push(event);
            return Ok(());
        }
        Err(ECSError::ResourceNotFound.into())
    }

    // Adds a system to react to events. Currently this just adds a system to last.
    // Eventually it will do something smarter probably.
    pub fn add_observer<E: Event>(&mut self, system: System) -> &mut Self {
        self.add_label_system(ScheduleLabel::Last, system)
    }

    pub fn exit(&mut self) {
        self.exit = true;
    }

    // Starts a loop to execute the scheduler.
    pub fn run(&mut self) {
        loop {
            if self.exit {
                self.scheduler.shutdown = true;
            }
            let mut s = self.scheduler.clone();
            if s.execute(self) {
                break;
            }
            self.scheduler.started = true;
        }
    }
}

// System to run event updates.
pub fn clear_events(world: &mut World) -> anyhow::Result<()> {
    for (_, update) in world.event_updates.clone().iter() {
        (update)(world)
    }
    Ok(())
}

// A label to group systems together in the scheduler.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ScheduleLabel {
    Start,    // Runs once on startup.
    First,    // Runs before any main update code.
    Update,   // Main app logic.
    Last,     // Runs after main update code.
    Clear,    // Clear state for next loop.
    Shutdown, // Run before exiting.
}

// A scheduler to do manual and automatic async for ECS
// systems. For now it just does basic ordering and grouping.
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

    // Builds the order of the ScheduleLabels into a graph.
    pub fn build_graph(&mut self) -> DiGraphMap<ScheduleLabel, ()> {
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

    // Run a loop of the configured schedule.
    pub fn execute(&mut self, world: &mut World) -> bool {
        let dag = self.build_graph();
        let start = dag.nodes().nth(0).unwrap();
        let mut bfs = Bfs::new(&dag, start);
        while let Some(label) = bfs.next(&dag) {
            if let Some(list) = self.schedule.get_mut(&label) {
                for system in list.iter() {
                    system(world).expect("systems don't support returning errors yet.");
                }
            }
        }
        self.shutdown
    }

    // Add a system to a group defined by label.
    pub fn add_system(&mut self, label: ScheduleLabel, system: System) {
        let list = self.schedule.entry(label).or_insert(Vec::new());
        list.push(system);
    }
}
