use alloc::collections::VecDeque;
use alloc::sync::Arc;
use core::cell::{Cell, RefCell};

use crate::sbi;
use crate::thread::{self, Thread, schedule};
use crate::sync::utils::get_thread_index;

/// Atomic counting semaphore
///
/// # Examples
/// ```
/// let sema = Semaphore::new(0);
/// sema.down();
/// sema.up();
/// ```
#[derive(Clone)]
pub struct Semaphore {
    value: Cell<usize>,
    waiters: RefCell<VecDeque<Arc<Thread>>>,
}

unsafe impl Sync for Semaphore {}
unsafe impl Send for Semaphore {}

impl Semaphore {
    /// Creates a new semaphore of initial value n.
    pub const fn new(n: usize) -> Self {
        Semaphore {
            value: Cell::new(n),
            waiters: RefCell::new(VecDeque::new()),
        }
    }

    /// P operation
    pub fn down(&self) {
        let old = sbi::interrupt::set(false);

        // Is semaphore available?
        while self.value() == 0 {
            // `push_front` ensures to wake up threads in a fifo manner
            self.waiters.borrow_mut().push_front(thread::current());

            // Block the current thread until it's awakened by an `up` operation
            thread::block();
        }
        self.value.set(self.value() - 1);

        sbi::interrupt::set(old);
    }

    /// V operation
    pub fn up(&self) {
        let _ = self.up_deferred();
    }

    /// V operation without immediate scheduling. Returns whether waking a
    /// higher-priority thread suggests a reschedule.
    pub fn up_deferred(&self) -> bool {
        let old = sbi::interrupt::set(false);
        let count = self.value.replace(self.value() + 1);
        let mut need_schedule = false;

        let idx = {
            let waiters = self.waiters.borrow();
            get_thread_index(&waiters)
        };

        if let Some(index) = idx {
            assert_eq!(count, 0);
            let thread = self.waiters.borrow_mut().remove(index).unwrap();
            need_schedule = thread.get_priority() > thread::current().get_priority();
            thread::wake_up(thread.clone());
        }

        sbi::interrupt::set(old);

        // Only schedule immediately when this function itself turned interrupts
        // off. If caller already had interrupts disabled, let caller decide a
        // safer scheduling point after its outer critical section.
        if need_schedule && old {
            schedule();
        }

        need_schedule
    }

    /// Get the current value of a semaphore
    pub fn value(&self) -> usize {
        self.value.get()
    }

    pub fn get_highest_waiter_priority(&self) -> u32 {
        self.waiters
            .borrow()
            .iter()
            .map(|thread| thread.get_priority())
            .max()
            .unwrap_or(0)
    }
}
