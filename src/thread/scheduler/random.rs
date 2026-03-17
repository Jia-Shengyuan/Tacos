use core::sync::atomic::{AtomicU64, Ordering::SeqCst};

use crate::sbi::timer;

// Lightweight PRNG for scheduler internals (e.g., tie-breaking).
// Not cryptographically secure.
static STATE: AtomicU64 = AtomicU64::new(0);

fn ensure_seeded() {
    if STATE.load(SeqCst) == 0 {
        // Mix timer clock and a fixed odd constant to avoid zero state.
        let seed = (timer::clock() as u64) ^ 0x9E3779B97F4A7C15;
        STATE.store(seed | 1, SeqCst);
    }
}

/// Returns a pseudo-random u32.
pub fn next_u32() -> u32 {
    ensure_seeded();

    // xorshift64*
    let mut x = STATE.load(SeqCst);
    x ^= x >> 12;
    x ^= x << 25;
    x ^= x >> 27;
    STATE.store(x, SeqCst);

    (x.wrapping_mul(0x2545F4914F6CDD1D) >> 32) as u32
}

/// Returns a pseudo-random number in [0, upper).
pub fn random_range(upper: usize) -> usize {
    if upper == 0 {
        return 0;
    }
    (next_u32() as usize) % upper
}
