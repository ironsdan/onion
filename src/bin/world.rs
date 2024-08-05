use hecs::Entity;
use onion::{
    ecs::{Event, EventBuffer, Resource, World},
    schedule::ScheduleLabel,
};
use onion_macros::{Event, Resource};
use std::time::Duration;

fn start_system(world: &mut World) -> anyhow::Result<()> {
    println!("this is a start up message and will run when the app first starts");
    world.inner.spawn((0, "p0", 10.0));
    world.inner.spawn((1, "p1", 5.0));
    world.inner.spawn((2, "p2", 5.0));
    world.inner.spawn((3, "p3", 5.0));
    world.inner.spawn((4, "p4", 10.0));
    Ok(())
}

fn name_system(world: &mut World) -> anyhow::Result<()> {
    for (_, (name, health)) in &mut world.inner.query::<(&&str, &mut f64)>() {
        println!("{} has {:.2}hp", name, health);
    }
    Ok(())
}

fn score_system(world: &mut World) -> anyhow::Result<()> {
    let score = world.get_resource::<GameState>().unwrap().score;
    println!("score: {}", score);
    if score == 5 {
        world.exit();
    }
    Ok(())
}

fn health_system(world: &mut World) -> anyhow::Result<()> {
    let mut deaths = Vec::new();
    for (_, (id, health)) in &mut world.inner.query::<(&mut i32, &mut f64)>() {
        if *health == 1.0 {
            deaths.push(*id);
        }
        *health = (*health) - (1.0);
    }
    for id in deaths {
        println!("trigger death");
        if world.trigger(DeathEvent { id }).is_none() {
            println!("tried to trigger but that event isn't registered.")
        }
    }

    Ok(())
}

fn death_observer(world: &mut World) -> anyhow::Result<()> {
    let ids: Vec<i32> = world
        .get_resource::<EventBuffer<DeathEvent>>()
        .unwrap()
        .events
        .iter()
        .map(|DeathEvent { id }| *id)
        .collect();
    if ids.len() == 0 {
        return Ok(());
    }

    for player_id in ids.iter() {
        println!("!!{:?} died!!", player_id);
        let mut gs = world.get_resource_mut::<GameState>().unwrap();
        gs.score += 1;
    }

    let entities = world
        .inner
        .query::<(&i32, &f64)>()
        .iter()
        .filter(|e| ids.contains(e.1 .0))
        .map(|e| e.0)
        .collect::<Vec<Entity>>();

    for e in entities {
        world.inner.despawn(e).unwrap();
    }

    Ok(())
}

fn sleep_system(_: &mut World) -> anyhow::Result<()> {
    std::thread::sleep(Duration::from_millis(250));
    Ok(())
}

fn shutdown_message(_: &mut World) -> anyhow::Result<()> {
    println!("goobye, thanks for playing! :)");
    Ok(())
}

#[derive(Resource, Debug)]
struct GameState {
    score: u32,
}

impl GameState {
    pub fn new() -> Self {
        Self { score: 0 }
    }
}

#[derive(Event, Default, Debug)]
struct DeathEvent {
    id: i32,
}

fn main() -> anyhow::Result<()> {
    let mut world = World::new();

    // Shows overwriting the resource.
    world.insert_resource(GameState::new());
    println!("{:?}", world.get_resource::<GameState>());
    world.insert_resource(GameState { score: 10 });
    println!("{:?}", world.get_resource::<GameState>());

    // Begins running the main loop. Which exits when the score
    // reaches 5 (see score_system).
    world
        .insert_resource(GameState::new())
        .add_label_system(ScheduleLabel::Start, start_system)
        .register_event::<DeathEvent>()
        .add_system(health_system)
        .add_system(name_system)
        .add_system(score_system)
        .add_observer::<DeathEvent>(death_observer)
        .add_label_system(ScheduleLabel::Last, sleep_system)
        .add_label_system(ScheduleLabel::Shutdown, shutdown_message)
        .run();

    Ok(())
}
