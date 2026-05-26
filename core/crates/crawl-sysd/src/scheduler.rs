//! # Crawl Scheduler
//!
//! Central scheduler with jitter for timing domain tasks.
//!
//! ## Architecture
//!
//! ```text
//! Scheduler ──► TaskRegistry ──► TaskState (per task)
//!                            ├── interval
//!                            ├── jitter
//!                            └── next_run
//! ```
//!
//! ## Usage
//!
//! Domains can optionally use the scheduler for timing:
//!
//! ```rust
//! use crawl_sysd::{Scheduler, ShouldRun};
//! use std::time::Duration;
//!
//! pub async fn run(cfg: Config, tx: Sender) -> Result<()> {
//!     run_with_scheduler(cfg, tx, None).await
//! }
//!
//! pub async fn run_with_scheduler(
//!     cfg: Config,
//!     tx: Sender,
//!     scheduler: Option<Scheduler>,
//! ) -> Result<()> {
//!     let task_id = "mytask";
//!     if let Some(ref sched) = scheduler {
//!         sched.register(task_id, Duration::from_secs(1), 0.1).await;
//!     }
//!
//!     loop {
//!         if let Some(ref sched) = scheduler {
//!             sched.wait_interval(task_id, 500).await;
//!             if sched.should_run(task_id).await == ShouldRun::Yes {
//!                 // do work
//!                 sched.mark_ran(task_id).await;
//!             } else {
//!                 continue;
//!             }
//!         } else {
//!             tokio::time::sleep(Duration::from_secs(1)).await;
//!         }
//!     }
//! }
//! ```
//!
//! ## Task Registration
//!
//! | Task | Interval | Jitter |
//! |------|----------|--------|
//! | sysmon | 1s | 5% |
//! | network_fast | 5s | 10% |
//! | network_slow | 30s | 15% |
//! | bluetooth | 5s | 10% |
//! | clipboard | 500ms | 10% |

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::Mutex;

#[cfg(feature = "sync")]
pub use blocking::BlockingScheduler;

#[derive(Clone)]
pub struct Scheduler {
    tasks: Arc<Mutex<HashMap<&'static str, TaskState>>>,
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn register(&self, id: &'static str, interval: Duration, jitter_pct: f32) {
        let state = TaskState::new(id, interval, jitter_pct);
        let mut tasks = self.tasks.lock().await;
        tasks.insert(id, state);
    }

    pub async fn unregister(&self, id: &'static str) {
        let mut tasks = self.tasks.lock().await;
        tasks.remove(id);
    }

    pub async fn should_run(&self, id: &'static str) -> ShouldRun {
        let tasks = self.tasks.lock().await;
        match tasks.get(id) {
            Some(state) => state.should_run(),
            None => ShouldRun::Yes,
        }
    }

    pub async fn mark_ran(&self, id: &'static str) {
        let mut tasks = self.tasks.lock().await;
        if let Some(state) = tasks.get_mut(id) {
            state.mark_ran();
        }
    }

    pub async fn reset(&self, id: &'static str) {
        let mut tasks = self.tasks.lock().await;
        if let Some(state) = tasks.get_mut(id) {
            state.reset();
        }
    }

    pub async fn wait_interval(&self, id: &'static str, poll_interval_ms: u64) {
        loop {
            match self.should_run(id).await {
                ShouldRun::Yes => return,
                ShouldRun::No => {
                    tokio::time::sleep(Duration::from_millis(poll_interval_ms)).await;
                }
            }
        }
    }

    pub async fn tasks(&self) -> Vec<&'static str> {
        let tasks = self.tasks.lock().await;
        tasks.keys().copied().collect()
    }

    pub async fn task_count(&self) -> usize {
        let tasks = self.tasks.lock().await;
        tasks.len()
    }
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShouldRun {
    Yes,
    No,
}

#[derive(Clone)]
pub struct TaskState {
    #[expect(dead_code)]
    pub id: &'static str,
    pub interval: Duration,
    pub jitter: Duration,
    pub next_run: Instant,
    pub last_run: Instant,
}

impl TaskState {
    pub fn new(id: &'static str, interval: Duration, jitter_pct: f32) -> Self {
        let jitter = Duration::from_secs_f64(interval.as_secs_f64() * jitter_pct as f64);
        let now = Instant::now();

        let initial_delay = if jitter.is_zero() {
            Duration::ZERO
        } else {
            Duration::from_secs_f64(fastrand::u64(0..=jitter.as_millis() as u64) as f64 / 1000.0)
        };

        Self {
            id,
            interval,
            jitter,
            next_run: now + initial_delay,
            last_run: now - interval,
        }
    }

    pub fn should_run(&self) -> ShouldRun {
        if Instant::now() >= self.next_run {
            ShouldRun::Yes
        } else {
            ShouldRun::No
        }
    }

    pub fn mark_ran(&mut self) {
        let now = Instant::now();
        let jitter_value = if self.jitter.is_zero() {
            Duration::ZERO
        } else {
            Duration::from_secs_f64(
                fastrand::u64(0..=self.jitter.as_millis() as u64) as f64 / 1000.0,
            )
        };
        self.last_run = now;
        self.next_run = now + self.interval + jitter_value;
    }

    pub fn reset(&mut self) {
        let now = Instant::now();
        self.next_run = now;
        self.last_run = now - self.interval;
    }
}

#[cfg(feature = "sync")]
pub mod blocking {
    use super::{ShouldRun, TaskState};
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    pub struct BlockingScheduler {
        tasks: Arc<Mutex<HashMap<&'static str, TaskState>>>,
    }

    impl BlockingScheduler {
        pub fn new() -> Self {
            Self {
                tasks: Arc::new(Mutex::new(HashMap::new())),
            }
        }

        pub fn register(&self, id: &'static str, interval: Duration, jitter_pct: f32) {
            let state = TaskState::new(id, interval, jitter_pct);
            let mut tasks = self.tasks.lock().unwrap();
            tasks.insert(id, state);
        }

        pub fn should_run(&self, id: &'static str) -> bool {
            let tasks = self.tasks.lock().unwrap();
            matches!(tasks.get(id).map(|t| t.should_run()), Some(ShouldRun::Yes))
        }

        pub fn mark_ran(&self, id: &'static str) {
            let mut tasks = self.tasks.lock().unwrap();
            if let Some(state) = tasks.get_mut(id) {
                state.mark_ran();
            }
        }

        pub fn wait_until_ready(&self, id: &'static str) {
            loop {
                if self.should_run(id) {
                    return;
                }
                std::thread::sleep(Duration::from_millis(100));
            }
        }
    }

    impl Default for BlockingScheduler {
        fn default() -> Self {
            Self::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_register() {
        let scheduler = Scheduler::new();
        scheduler
            .register("test", Duration::from_secs(1), 0.1)
            .await;
        assert_eq!(scheduler.task_count().await, 1);
    }

    #[tokio::test]
    async fn test_should_run() {
        let scheduler = Scheduler::new();
        scheduler
            .register("test", Duration::from_secs(1), 0.1)
            .await;
        assert!(matches!(scheduler.should_run("test").await, ShouldRun::Yes));
    }

    #[tokio::test]
    async fn test_mark_ran() {
        let scheduler = Scheduler::new();
        scheduler
            .register("test", Duration::from_secs(1), 0.0)
            .await;
        scheduler.mark_ran("test").await;
        assert!(matches!(scheduler.should_run("test").await, ShouldRun::No));
    }

    #[tokio::test]
    async fn test_unregister() {
        let scheduler = Scheduler::new();
        scheduler
            .register("test", Duration::from_secs(1), 0.1)
            .await;
        scheduler.unregister("test").await;
        assert_eq!(scheduler.task_count().await, 0);
    }

    #[test]
    fn test_task_state_initial() {
        let task = TaskState::new("test", Duration::from_secs(1), 0.1);
        assert!(matches!(task.should_run(), ShouldRun::Yes));
    }

    #[test]
    fn test_task_state_after_mark_ran() {
        let mut task = TaskState::new("test", Duration::from_secs(1), 0.0);
        task.mark_ran();
        assert!(matches!(task.should_run(), ShouldRun::No));
    }
}
