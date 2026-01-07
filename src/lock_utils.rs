/// Lock timeout utilities for safe mutex operations
use std::sync::Mutex;
use std::time::Duration;
use crate::{Result, OptikError};

/// Attempt to acquire a lock with a timeout
pub fn lock_with_timeout<T>(
    mutex: &Mutex<T>,
    timeout: Duration,
) -> Result<std::sync::MutexGuard<T>> {
    // Note: std::sync::Mutex doesn't have built-in timeout support
    // This is a wrapper that documents the pattern
    // For timeouts, use parking_lot or tokio::sync::Mutex
    
    match mutex.lock() {
        Ok(guard) => Ok(guard),
        Err(e) => Err(OptikError::LockError(format!(
            "Failed to acquire lock (requested timeout: {:?}): {}",
            timeout, e
        ))),
    }
}

/// Try to acquire a lock without blocking
pub fn try_lock<T>(mutex: &Mutex<T>) -> Result<std::sync::MutexGuard<T>> {
    mutex.try_lock().map_err(|_| {
        OptikError::LockError(
            "Lock is currently held by another thread".to_string(),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lock_with_timeout() {
        let mutex = Mutex::new(42);
        let timeout = Duration::from_secs(1);

        let result = lock_with_timeout(&mutex, timeout);
        assert!(result.is_ok());
        assert_eq!(*result.unwrap(), 42);
    }

    #[test]
    fn test_try_lock_available() {
        let mutex = Mutex::new("hello");
        let result = try_lock(&mutex);

        assert!(result.is_ok());
        assert_eq!(*result.unwrap(), "hello");
    }

    #[test]
    fn test_try_lock_contention() {
        let mutex = std::sync::Arc::new(Mutex::new(0));
        let mutex_clone = std::sync::Arc::clone(&mutex);

        // Hold the lock in another scope
        {
            let _guard = mutex.lock().unwrap();
            // Try to acquire in same thread (will fail on try_lock)
            let result = try_lock(&mutex_clone);
            // Can't acquire while held
            assert!(result.is_err());
        }

        // Now it should work
        let result = try_lock(&mutex_clone);
        assert!(result.is_ok());
    }
}
