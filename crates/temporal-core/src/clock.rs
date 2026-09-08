//! Capture an injected clock once; all later calculations use that value.

use crate::time::{Instant, TimeError, TimedSpan};
use std::cell::Cell;

pub trait Clock {
    fn now(&self) -> Instant;
}

#[derive(Debug)]
pub struct FrozenClock {
    instant: Cell<Instant>,
}

impl FrozenClock {
    pub fn new(instant: Instant) -> Self {
        Self {
            instant: Cell::new(instant),
        }
    }
    pub fn advance_ms(&self, milliseconds: u64) -> Result<(), TimeError> {
        let next = self.instant.get().checked_add_ms(milliseconds)?;
        self.instant.set(next);
        Ok(())
    }
}

impl Clock for FrozenClock {
    fn now(&self) -> Instant {
        self.instant.get()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvaluationTime {
    span: TimedSpan,
}

impl EvaluationTime {
    pub fn capture(
        captured_now: Instant,
        evaluation_end: Instant,
        clock: &impl Clock,
    ) -> Result<Self, TimeError> {
        let now = clock.now();
        Self::new(captured_now, now, evaluation_end)
    }
    pub fn new(
        captured_now: Instant,
        now: Instant,
        evaluation_end: Instant,
    ) -> Result<Self, TimeError> {
        if now < captured_now {
            return Err(TimeError::EvaluationBeforeSnapshot);
        }
        Ok(Self {
            span: TimedSpan::new(now, evaluation_end)?,
        })
    }
    pub fn now(&self) -> Instant {
        self.span.start()
    }
    pub fn evaluation_end(&self) -> Instant {
        self.span.end()
    }
    pub fn span(&self) -> &TimedSpan {
        &self.span
    }
}
