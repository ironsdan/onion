use std::collections::LinkedList;

pub fn net() {
    println!("test")
}

pub struct Replayable<Input, State> {
    next_fn: fn(&Input, &State) -> State,

    // The frame number of the last state
    frame: u64,
    // History of inputs. The first entry corresponds to the first state. It should be read as the
    // input to be applied to the first state to get the subsequent state.
    history: LinkedList<Input>,
    // State of the oldest frame in the buffer. When we receive inputs on an old frame, we recompute
    // all frames since the first in order to generate the last
    first: State,
    // The last frame. This is kept as a cache so we don't need to repeatedly recompute the frame
    last: State,
    // Indicates the last frame is out of date and will need recomputation next time it is accessed.
    stale: bool
}


impl <Input: Clone, State: Clone> Replayable<Input, State> {
    pub fn new(next: fn(&Input, &State) -> State, seed: State, input: Input) -> Replayable<Input, State> {
        let mut history = LinkedList::new();
        history.push_front(input);
        return Replayable {
            next_fn: next,
            frame: 1,
            history: history,
            first: seed.clone(),
            last: seed.clone(),
            stale: false,
        }
    }

    // Forces a particular frame to have the given inputs and state. In the process, any inputs
    // and state from prior frames is erased. If the requested force frame is older than the
    // history buffer, the force will be ignored.
    pub fn force(&mut self, id: u64, input: Input, state: State) {
        // this is an important optimization. When joining a game you might be forced forward
        // millions of frames. if you have to compute them pointlessly, that would be a waste.
        let server_ahead = id > self.frame;
        if server_ahead {
            self.frame = id;
            self.history.clear();
            self.history.push_back(input);
            self.first = state.clone();
            self.last = state.clone();
            self.stale = false;
            return;
        }

        // We don't execute commits unless the server tells us to. That means this is an outdated
        // message. We should just ignore it.
        let server_behind = id < (self.frame - self.history.len() as u64);
        if server_behind {
            return
        }

        self.commit(id);
        self.stale = true;
        self.first = state;
        self.history.pop_front();
        self.history.push_front(input);
    }

    pub fn current(&mut self) -> &State {
        if !self.stale {
            return &self.last;
        }
        self.last = self.first.clone();
        for frame in self.history.iter() {
            self.last = (self.next_fn)(frame, &self.last);
        }
        self.stale = false;
        return &self.last;
    }

    // Recomputes until on the desired frame
    pub fn fast_forward(&mut self, frame: u64) {
        for _i in self.frame..frame {
            self.advance(self.history.back().unwrap().clone())
        }
    }

    // Creates a new frame (does not compute the frame)
    pub fn advance(&mut self, input: Input) {
        self.stale = true;
        self.history.push_back(input);
        self.frame+= 1;
    }

    // Commits all frames before the given id, clearing them from the buffer
    pub fn commit(&mut self, id: u64) {
        let missing = id - self.frame;
        for _i in 0..missing {
            self.advance(self.history.back().unwrap().clone())
        }
        let start = self.frame - self.history.len() as u64;
        for _i in 0..(id - start) {
            let frame = self.history.pop_front();
            self.first = (self.next_fn)(frame.as_ref().unwrap(), &self.first)
        }
    }


    // Update an already existing input. If the frame is afgter the latest frame, the buffer will
    // be advanced until the frames match. Newly created frames will copy the input of their prior
    // frame. If the id is beyond the range of the buffer, nothing will happen.
    pub fn update_input(&mut self, id: u64, apply: fn(&mut Input)) {
        let missing = (id as i64) - (self.frame as i64);
        for _i in 0..missing {
            self.advance(self.history.back().unwrap().clone())
        }

        self.stale = true;
        let mut iter = self.history.iter_mut();
        for _i in 0..(self.frame - id) {
            let n = iter.next();
            if n.is_none() {
                return
            }
        }
        let input = iter.next();
        if input.is_none() {
            return
        }
        apply(input.unwrap());
    }
}