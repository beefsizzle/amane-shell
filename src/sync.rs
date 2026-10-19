use std::sync::{Mutex, MutexGuard, PoisonError};

/// locks a mutex whatever happened to the last thread that held it. everything these mutexes guard is a plain
/// list or number, so a thread that panicked halfway can't have left it in a state worth refusing to read
pub fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}
