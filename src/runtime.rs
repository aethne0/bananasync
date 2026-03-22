use std::{
    cell::RefCell,
    cmp::Reverse,
    collections::{BinaryHeap, HashMap, VecDeque},
    pin::Pin,
    sync::Arc,
    task::{Context, Poll, Waker},
    time::Instant,
};

struct Task {
    id: u64,
    future: Pin<Box<dyn Future<Output = ()> + 'static>>,
}

struct TaskWaker {
    task_id: u64,
}

impl std::task::Wake for TaskWaker {
    fn wake(self: Arc<Self>) {
        RUNTIME_LOCAL.with_borrow_mut(|rt| {
            if let Some(task) = rt.slept.remove(&self.task_id) {
                rt.queued.push_back(task);
            }
        });
    }
}

/// A thread-local runtime
pub struct Runtime {
    next_id_task: u64,

    queued: VecDeque<Task>,
    slept: HashMap<u64, Task>,

    next_id_event: u64,

    timers: BinaryHeap<Reverse<(Instant, u64)>>,
    timer_wakers: HashMap<u64, Waker>,
}

thread_local! {
    /// Thread-local runtime struct
    pub(crate) static RUNTIME_LOCAL: RefCell<Runtime> = RefCell::new(
        Runtime {
            next_id_task: 1,
            next_id_event: 1,
            queued: VecDeque::new(),
            slept: HashMap::new(),
            timers: BinaryHeap::new(),
            timer_wakers: HashMap::new(),
        }
    );
}

impl Runtime {
    pub fn spawn<F: Future<Output = ()> + 'static>(future: F) {
        RUNTIME_LOCAL.with_borrow_mut(|rt| {
            rt.queued.push_back(Task {
                id: rt.next_id_task,
                future: Box::pin(future),
            });
        });
    }

    pub fn block_on<F: Future<Output = ()> + 'static>(future: F) {
        Runtime::spawn(future);

        loop {
            let mut polled_any = false;

            while let Some(mut task) = RUNTIME_LOCAL.with_borrow_mut(|rt| rt.queued.pop_front()) {
                polled_any = true;
                let task_id = task.id;

                let waker: Waker = Arc::new(TaskWaker { task_id }).into();
                let mut cx = Context::from_waker(&waker);

                match task.future.as_mut().poll(&mut cx) {
                    Poll::Pending => {
                        RUNTIME_LOCAL.with_borrow_mut(|rt| {
                            rt.slept.insert(task_id, task);
                            rt.slept.drain().for_each(|(_, t)| {
                                rt.queued.push_back(t);
                            });
                        });
                    }

                    Poll::Ready(val) => {
                        return val;
                    }
                }

                break;
            }

            if polled_any {
                continue;
            }

            let waker = RUNTIME_LOCAL.with_borrow_mut(|rt| {
                if rt.slept.is_empty() {
                    return None;
                }

                let most_soon_timer = rt.timers.peek().expect(
                    "only timers are implemented - presence of slept task must mean theres a timer",
                ).0.0;

                std::thread::sleep(most_soon_timer.duration_since(Instant::now()));

                let event_id = rt.timers.pop().unwrap().0 .1;
                rt.timer_wakers.remove(&event_id)
            });

            if let Some(waker) = waker {
                waker.wake();
            }
        }
    }

    // TODO make above on self, and add init function

    pub(crate) fn register_timer(&mut self, deadline: Instant, waker: Waker) {
        self.timers.push(Reverse((deadline, self.next_id_event)));
        self.timer_wakers.insert(self.next_id_event, waker);
        self.next_id_event += 1;
    }
}
