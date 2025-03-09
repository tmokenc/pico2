use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Default, Clone)]
/// A sleep state that can be shared between cores.
pub struct SleepState(Rc<AtomicBool>);

impl SleepState {
    pub fn new(is_sleeping: bool) -> Self {
        Self(Rc::new(AtomicBool::new(is_sleeping)))
    }

    pub fn is_sleeping(&self) -> bool {
        self.0.load(Ordering::Relaxed)
    }

    pub fn wake(&self) {
        self.0.store(false, Ordering::Relaxed);
    }

    pub fn sleep(&self) {
        self.0.store(true, Ordering::Relaxed);
    }
}
