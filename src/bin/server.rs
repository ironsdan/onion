use std::time::Instant;
use std::time::Duration;
use onion::netcode::replay;

const time_per_tick: Duration = Duration::new(0, 13000000); // roughly 60 fps

fn main() {
    let mut adder = replay::Replayable::new(|i: &i64, s: &i64| -> i64 {i + s}, 0, 0);
    let start_time = Instant::now();
    let mut last_commit = 0;
    loop {
        let now = Instant::now();
        let tick = ((now - start_time).as_millis() / time_per_tick.as_millis()) as u64;
        adder.fast_forward(tick);
        if tick - last_commit > 15 {
            adder.commit(tick - 5);
            last_commit = tick - 5;
        }


    }
}
