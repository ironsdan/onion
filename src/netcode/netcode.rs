use std::collections::LinkedList;

pub fn net() {
    println!("test")
}

pub struct GameBuffer<Input, State> {
    frames: LinkedList<Frame<Input>>,
    // State of the oldest frame in the buffer. When we receive inputs on an old frame, we recompute
    // all frames since the first in order to generate the last
    first: State,
    // The last frame. This is kept as a cache so we don't need to repeatedly recompute the frame
    last: State,
    // Indicates the last frame is out of date and will need recomputation next time it is accessed.
    stale: bool
}


// A frame is just a set of inputs (that have not yet been applied) and an id number for that frame
pub struct Frame<Input> {
    id: u64,
    input: Input,
}