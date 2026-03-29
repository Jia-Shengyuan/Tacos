//! Kernel Threads

mod imp;
pub mod manager;
pub mod scheduler;
pub mod switch;

pub use self::imp::*;
pub use self::manager::Manager;
pub(self) use self::scheduler::{Schedule, Scheduler};

use crate::sync::Lazy;
use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use alloc::vec::Vec;

// Global sleep queue: wake_tick -> threads to wake at that tick.
pub static SLEEP_QUEUE: Lazy<Mutex<BTreeMap<i64, Vec<Arc<Thread>>>>> =
    Lazy::new(|| Mutex::new(BTreeMap::new()));

/// Create a new thread
pub fn spawn<F>(name: &'static str, f: F) -> Arc<Thread>
where
    F: FnOnce() + Send + 'static,
{
    Builder::new(f).name(name).spawn()
}

/// Get the current running thread
pub fn current() -> Arc<Thread> {
    Manager::get().current.lock().clone()
}

/// Yield the control to another thread (if there's another one ready to run).
pub fn schedule() {
    Manager::get().schedule()
}

/// Gracefully shut down the current thread, and schedule another one.
pub fn exit() -> ! {
    {
        let current = Manager::get().current.lock();

        #[cfg(feature = "debug")]
        kprintln!("Exit: {:?}", *current);

        current.set_status(Status::Dying);
    }

    schedule();

    unreachable!("An exited thread shouldn't be scheduled again");
}

/// Mark the current thread as [`Blocked`](Status::Blocked) and
/// yield the control to another thread
pub fn block() {
    let current = current();
    current.set_status(Status::Blocked);

    #[cfg(feature = "debug")]
    kprintln!("[THREAD] Block {:?}", current);

    schedule();
}

/// Wake up a previously blocked thread, mark it as [`Ready`](Status::Ready),
/// and register it into the scheduler.
pub fn wake_up(thread: Arc<Thread>) {
    assert_eq!(thread.status(), Status::Blocked);
    thread.set_status(Status::Ready);

    #[cfg(feature = "debug")]
    kprintln!("[THREAD] Wake up {:?}", thread);

    Manager::get().scheduler.lock().register(thread);
}

/// (Lab1) Sets the current thread's priority to a given value
pub fn set_priority(_priority: u32) {
    // Do not wrap this path with interrupt::set(false): schedule() may switch
    // to another thread immediately, and interrupt state is hart-global here.
    // If we switch out while interrupts are disabled, the next thread may run
    // in an unintended interrupt-off window and cause timer-based timeouts.
    current().set_priority(_priority);
    schedule();
}

/// (Lab1) Returns the current thread's effective priority.
pub fn get_priority() -> u32 {
    current().get_priority()
}

// check blocked threads in the sleep queue to see who to wake up
pub fn check_wakeup() {
    use crate::sbi::timer::timer_ticks;

    let now = timer_ticks();
    let mut expired = Vec::new();

    {
        let mut queue = SLEEP_QUEUE.lock();
        loop {
            let Some((&wake_tick, _)) = queue.iter().next() else {
                break;
            };
            if wake_tick > now {
                break;
            }
            if let Some(mut threads) = queue.remove(&wake_tick) {
                expired.append(&mut threads);
            }
        }
    }

    for thread in expired {
        wake_up(thread);
    }
}

/// (Lab1) Make the current thread sleep for the given ticks.
pub fn sleep(ticks: i64) {

    use crate::sbi::{interrupt, timer::timer_ticks};
    if ticks <= 0 { return; }

    let old = interrupt::set(false);
    let wake_tick = timer_ticks() + ticks;
    
    SLEEP_QUEUE.lock().entry(wake_tick).or_default().push(current());

    block();
    interrupt::set(old);
}
