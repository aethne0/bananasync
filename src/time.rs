use std::{
    ops::Add,
    pin,
    task::{Context, Poll},
    time::{Duration, Instant},
};

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

    fn poll(mut self: pin::Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        // If a zero duration was passed in we can be Ready immediately
        if self.duration.is_zero() {
            return Poll::Ready(());
        }

        if let Some(deadline) = self.deadline {
            if Instant::now() >= deadline {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        } else {
            self.deadline = Some(Instant::now().add(self.duration));
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

