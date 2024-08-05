use super::schedule::{ScheduleLabel, Scheduler};
use hecs::Entity;
use std::any::TypeId;
use std::collections::hash_map::Entry::Vacant;
use std::collections::HashMap;
use std::fmt::Debug;

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
        if let Some(_) = self.resource_map.get(&TypeId::of::<R>()) {
            self.delete_resource::<R>().expect(
                "tried to delete resource that doesn't exit, even though it exists in resource_map",
            );
        }
        let e = self.inner.spawn((resource,));
        self.resource_map.insert(TypeId::of::<R>(), e);
        self
    }

    // Return a unique reference to a resource or an error if it didn't exist.
    pub fn get_resource_mut<R: Resource>(&mut self) -> Option<hecs::RefMut<'_, R>> {
        let e = self.resource_map.get(&TypeId::of::<R>())?;
        match self.inner.get::<&mut R>(*e) {
            Ok(tmp) => Some(tmp),
            Err(_) => None,
        }
    }

    // Return a shared reference to a resource or an error if it didn't exist.
    pub fn get_resource<R: Resource>(&mut self) -> Option<hecs::Ref<'_, R>> {
        let e = self.resource_map.get(&TypeId::of::<R>())?;
        match self.inner.get::<&R>(*e) {
            Ok(tmp) => Some(tmp),
            Err(_) => None,
        }
    }

    // Delete a resource or an error if it didn't exist.
    pub fn delete_resource<R: Resource>(&mut self) -> anyhow::Result<()> {
        let e = self.resource_map.get(&TypeId::of::<R>()).unwrap();
        self.inner.get::<&R>(*e)?;
        self.inner.despawn(*e)?;
        Ok(())
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
    // event will return an error.
    pub fn deregister_event<E: Event>(&mut self) -> anyhow::Result<()> {
        self.delete_resource::<EventBuffer<E>>()?;
        self.event_updates.remove(&TypeId::of::<EventBuffer<E>>());
        Ok(())
    }

    // Send an event to the event queue for observers to view react to.
    pub fn trigger<E: Event>(&mut self, event: E) -> Option<()> {
        self.get_resource_mut::<EventBuffer<E>>()?
            .events
            .push(event);
        Some(())
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
