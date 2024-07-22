use onion::app::App;
use std::{error::Error, time::Duration};

fn death_system(world: &mut hecs::World) -> Result<(), Box<dyn Error>> {
    for (_, health) in &mut world.query::<&mut f64>() {
        *health = (*health) - (0.1);
    }

    Ok(())
}

fn name_system(world: &mut hecs::World) -> Result<(), Box<dyn Error>> {
    for (_, (name, health)) in &mut world.query::<(&&str, &mut f64)>() {
        println!("{} has {:.2}hp", name, health);
    }
    Ok(())
}

fn sleep_system(_: &mut hecs::World) -> Result<(), Box<dyn Error>> {
    std::thread::sleep(Duration::from_secs(1));
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut app = App::new();
    app.world.spawn(("p1", 100.0));
    app.world.spawn(("p2", 50.0));
    app.add_system(Box::new(death_system))
        .add_system(Box::new(name_system))
        .add_system(Box::new(sleep_system))
        .run();
    Ok(())
}
