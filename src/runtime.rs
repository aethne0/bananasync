use std::{
    cell::RefCell,
    collections::VecDeque,
    pin::Pin,
    task::{Context, Poll, Waker},
};

struct Task {
    id: u64,
    future: Pin<Box<dyn Future<Output = ()> + 'static>>,
}

/// A thread-local runtime
pub struct Runtime {
    next_id: u64,
    queued: VecDeque<Task>,
    slept: VecDeque<Task>,
}

thread_local! {
    /// Thread-local runtime struct
    static THREAD_RT: RefCell<Runtime> = RefCell::new(Runtime { next_id: 1, queued: VecDeque::new(),slept: VecDeque::new()});
}

impl Runtime {
    pub fn spawn<F: Future<Output = ()> + 'static>(future: F) {
        THREAD_RT.with_borrow_mut(|rt| {
            rt.queued.push_back(Task {
                id: rt.next_id,
                future: Box::pin(future),
            });
        });
    }

    pub fn block_on<F: Future<Output = ()> + 'static>(future: F) {
        Runtime::spawn(future);
        let mut cx = Context::from_waker(Waker::noop());

        loop {
            loop {
                if let Some(mut task) = THREAD_RT.with_borrow_mut(|rt| rt.queued.pop_front()) {
                    match task.future.as_mut().poll(&mut cx) {
                        Poll::Pending => {
                            THREAD_RT.with_borrow_mut(|rt| {
                                rt.slept.push_back(task);
                            });
                        }
                        Poll::Ready(val) => {
                            return val;
                        }
                    }
                } else {
                    break;
                }
            }

            THREAD_RT.with_borrow_mut(|rt| {
                if rt.slept.is_empty() {
                    return;
                }

                // temporary busy loop - simply plop all slep tasks into queeu
                std::mem::swap(&mut rt.queued, &mut rt.slept);
            });
        }
    }
}
