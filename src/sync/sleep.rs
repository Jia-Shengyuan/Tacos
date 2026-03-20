use alloc::sync::Arc;
use core::cell::RefCell;

use crate::sbi;
use crate::sync::{Lock, Semaphore};
use crate::thread::{self, Thread};

/// Sleep lock. Uses [`Semaphore`] under the hood.
#[derive(Clone)]
pub struct Sleep {
    inner: Semaphore,
    holder: RefCell<Option<Arc<Thread>>>,
}

impl Default for Sleep {
    fn default() -> Self {
        Self {
            inner: Semaphore::new(1),
            holder: Default::default(),
        }
    }
}

impl Sleep {
    fn lock_id(&self) -> usize {
        self as *const Self as usize
    }
    pub fn get_highest_waiter_priority(&self) -> u32 {
        self.inner.get_highest_waiter_priority()
    }
}

impl Lock for Sleep {
    fn acquire(&self) {
        let old = sbi::interrupt::set(false);
        let current = thread::current();
        if self.holder.borrow().is_some() {
            current.set_waiting_lock(self.lock_id());
        }
        self.inner.down();

        let current = thread::current();
        current.clear_waiting_lock();
        current.add_held_lock(self.lock_id());
        self.holder.borrow_mut().replace(current);
        sbi::interrupt::set(old);
    }

    fn release(&self) {
        let old = sbi::interrupt::set(false);
        assert!(Arc::ptr_eq(
            self.holder.borrow().as_ref().unwrap(),
            &thread::current()
        ));

        thread::current().remove_held_lock(self.lock_id());
        self.holder.borrow_mut().take().unwrap();
        let need_schedule = self.inner.up_deferred();
        sbi::interrupt::set(old);
        if need_schedule && old {
            thread::schedule();
        }
    }
}

unsafe impl Sync for Sleep {}
