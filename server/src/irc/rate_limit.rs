use std::sync::Arc;

use tokio::sync::{AcquireError, Semaphore};
use tokio::time::MissedTickBehavior;
use tokio::time::{Duration, interval};

pub struct Bucket {
    sem: Arc<Semaphore>,
    handle: tokio::task::JoinHandle<()>,
}

impl Bucket {
    pub fn new(duration: Duration, capacity: usize) -> Self {
        let sem = Arc::new(Semaphore::new(capacity));
        let handle = tokio::spawn({
            let sem = sem.clone();
            let mut interval = interval(duration);

            interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
            async move {
                loop {
                    interval.tick().await;
                    if sem.available_permits() < capacity {
                        sem.add_permits(1);
                    }
                }
            }
        });

        Self { sem, handle }
    }

    pub async fn acquire_one(&self) -> Result<bool, AcquireError> {
        let permit = self.sem.acquire().await?;
        permit.forget();

        Ok(true)
    }

    pub fn try_acquire_one(&self) -> bool {
        match self.sem.try_acquire() {
            Ok(_) => true,
            Err(_) => false,
        }
    }
}

impl Drop for Bucket {
    fn drop(&mut self) {
        self.handle.abort();
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test(start_paused = true)]
    async fn acquire_await_succeeds_when_permit() {
        let limiter = Bucket::new(Duration::from_millis(1500), 5);
        let acq = limiter.acquire_one().await.unwrap();

        assert!(acq);
    }

    #[tokio::test(start_paused = true)]
    async fn acquire_succeeds_when_permit() {
        let limiter = Bucket::new(Duration::from_millis(1500), 5);
        assert!(limiter.try_acquire_one());
    }
}
