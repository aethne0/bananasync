use std::{
    cell::RefCell,
    cmp::Reverse,
    collections::{BinaryHeap, HashMap, VecDeque},
    ops::Sub,
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
            let task = rt
                .as_mut()
                .expect("wake called outside runtime")
                .slept
                .remove(&self.task_id)
                .expect("woke with no slept tasks?");

            rt.as_mut().unwrap().queued.push_back(task);
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

impl Runtime {
    #[must_use]
    fn new() -> Self {
        Runtime {
            next_id_task: 1,
            next_id_event: 1,
            queued: VecDeque::new(),
            slept: HashMap::new(),
            timers: BinaryHeap::new(),
            timer_wakers: HashMap::new(),
        }
    }
}

thread_local! {
    /// Thread-local runtime struct
    pub(crate) static RUNTIME_LOCAL: RefCell<Option<Runtime>> = RefCell::new(None);
}

impl Runtime {
    pub fn spawn<F: Future<Output = ()> + 'static>(future: F) {
        RUNTIME_LOCAL.with_borrow_mut(|rt_borrow_mut| {
            let rt = rt_borrow_mut
                .as_mut()
                .expect("called spawn from outside runtime!");

            rt.queued.push_back(Task {
                id: rt.next_id_task,
                future: Box::pin(future),
            });

            rt.next_id_task += 1;
        });
    }

    pub fn block_on<F: Future<Output = ()> + 'static>(future: F) {
        RUNTIME_LOCAL.with(|rt_tls| {
            {
                // intialize thread local runtime
                let mut rt_borrow = rt_tls.borrow_mut();
                if rt_borrow.is_some() {
                    panic!("block_on called from inside runtime!");
                }
                *rt_borrow = Some(Runtime::new());
            }

            Runtime::block_on_inner(future);

            rt_tls.borrow_mut().take();
        });
    }

    pub fn block_on_inner<F: Future<Output = ()> + 'static>(future: F) {
        // note: runtime is definitely Some for the duration of this function, we will just unwrap
        RUNTIME_LOCAL.with(|rt_tls| {
            Runtime::spawn(future);

            loop {
                // {} because we want the expression to be droped, we are moving the task out and
                // we dont want to keep holding the RefCell mut borrow (while-let isnt a temporary
                // scope)
                while let Some(mut task) = {rt_tls.borrow_mut().as_mut().unwrap().queued.pop_front()} {
                    let waker: Waker = Arc::new(TaskWaker { task_id: task.id }).into();
                    let mut cx = Context::from_waker(&waker);

                    match task.future.as_mut().poll(&mut cx) {
                        Poll::Pending => {
                            rt_tls.borrow_mut().as_mut().unwrap().slept.insert(task.id, task);
                        }

                        Poll::Ready(val) => {
                            return val;
                        }
                    }
                }

                let waker = {
                    let mut rt_borrow = rt_tls.borrow_mut();
                    let rt_ref = rt_borrow.as_mut().unwrap();

                    if rt_ref.slept.is_empty() {
                        None
                    } else {
                        let most_soon_timer = rt_ref.timers.peek()
                            .expect("only timers are implemented - presence of slept task must mean theres a timer")
                            .0.0;

                        std::thread::sleep(most_soon_timer.sub(Instant::now()));

                        let event_id = rt_ref.timers.pop().unwrap().0.1;
                        rt_ref.timer_wakers.remove(&event_id)
                    }
                };

                if let Some(waker) = waker {
                    waker.wake();
                } else if  {
                    let queue_empty = rt_tls.borrow_mut().as_ref().unwrap().queued.is_empty();
                    let sleep_empty = rt_tls.borrow_mut().as_ref().unwrap().slept.is_empty();

                    queue_empty && sleep_empty
                }
                {
                    // if we didnt wake anything we check if queued && slept are,
                    // if so runtime is done executing.
                    break;
                }
            }

        });
    }

    // TODO make above on self, and add init function

    pub(crate) fn register_timer(&mut self, deadline: Instant, waker: Waker) {
        self.timers.push(Reverse((deadline, self.next_id_event)));
        self.timer_wakers.insert(self.next_id_event, waker);
        self.next_id_event += 1;
    }
}
