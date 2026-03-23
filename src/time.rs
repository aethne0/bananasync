use std::{
    ops::Add,
    pin,
    task::{Context, Poll},
    time::{Duration, Instant},
};

use crate::runtime::RUNTIME_LOCAL;

/// Snoozer is an `impl Future` that starts a timer when it is first polled.
pub struct SnoozeFut {
    duration: Duration,
    deadline: Option<Instant>,
}

impl SnoozeFut {
    #[must_use]
    fn new(duration: Duration) -> Self {
        Self {
            duration,
            deadline: None,
        }
    }
}

impl Future for SnoozeFut {
    type Output = ();

    fn poll(mut self: pin::Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // If a zero duration was passed in we can be Ready immediately
        if self.duration.is_zero() {
            return Poll::Ready(());
        }

        let duration = self.duration; // no-borrow
        let deadline = *self.deadline.get_or_insert(Instant::now().add(duration));

        if Instant::now() >= deadline {
            Poll::Ready(())
        } else {
            RUNTIME_LOCAL.with_borrow_mut(|rt| {
                rt.as_mut()
                    .unwrap()
                    .register_timer(deadline, cx.waker().clone());
            });
            Poll::Pending
        }
    }
}

/// Sometimes it is pertinent to take a snooze.
/// This starts the timer when it is first polled (`.await`ed). Constructing the future does not
/// start the timer.
/// The timer expires at the Instant `.await` is called plus the passed duration (approximately)
pub fn snooze(duration: Duration) -> SnoozeFut {
    SnoozeFut::new(duration)
}
