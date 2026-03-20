use std::{
    collections::VecDeque,
    pin,
    task::{Context, Poll, Waker},
};

pub struct Task {}

pub struct Runtime {
    queued: VecDeque<Task>,
    slept: VecDeque<Task>,
}

impl Runtime {
    #[must_use]
    pub fn new() -> Self {
        Runtime {
            queued: VecDeque::new(),
            slept: VecDeque::new(),
        }
    }

    pub fn block_on<F: Future>(&mut self, future: F) -> F::Output {
        let mut task = pin::pin!(future);
        let mut cx = Context::from_waker(Waker::noop());

        loop {
            match task.as_mut().poll(&mut cx) {
                Poll::Pending => {}
                Poll::Ready(val) => {
                    return val;
                }
            }
        }
    }
}
