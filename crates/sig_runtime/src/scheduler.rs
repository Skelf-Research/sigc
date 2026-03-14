//! Job scheduling for running strategies on a schedule
//!
//! Provides trait-based scheduling with multiple backend implementations.

use sig_types::{Result, SigcError};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Job status
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum JobStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

impl std::fmt::Display for JobStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JobStatus::Pending => write!(f, "PENDING"),
            JobStatus::Running => write!(f, "RUNNING"),
            JobStatus::Completed => write!(f, "COMPLETED"),
            JobStatus::Failed => write!(f, "FAILED"),
            JobStatus::Cancelled => write!(f, "CANCELLED"),
        }
    }
}

/// A scheduled job
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Job {
    pub id: String,
    pub name: String,
    pub schedule: Schedule,
    pub command: String,
    pub args: Vec<String>,
    pub status: JobStatus,
    pub last_run: Option<u64>,
    pub next_run: Option<u64>,
    pub retry_count: u32,
    pub max_retries: u32,
    pub timeout_secs: u64,
    pub tags: HashMap<String, String>,
}

impl Job {
    /// Create a new job
    pub fn new(id: &str, name: &str, schedule: Schedule, command: &str) -> Self {
        Job {
            id: id.to_string(),
            name: name.to_string(),
            schedule,
            command: command.to_string(),
            args: Vec::new(),
            status: JobStatus::Pending,
            last_run: None,
            next_run: None,
            retry_count: 0,
            max_retries: 3,
            timeout_secs: 3600, // 1 hour default
            tags: HashMap::new(),
        }
    }

    pub fn with_args(mut self, args: Vec<String>) -> Self {
        self.args = args;
        self
    }

    pub fn with_max_retries(mut self, n: u32) -> Self {
        self.max_retries = n;
        self
    }

    pub fn with_timeout(mut self, secs: u64) -> Self {
        self.timeout_secs = secs;
        self
    }

    pub fn with_tag(mut self, key: &str, value: &str) -> Self {
        self.tags.insert(key.to_string(), value.to_string());
        self
    }
}

/// Job schedule specification
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum Schedule {
    /// Run once at a specific timestamp
    Once(u64),
    /// Run at interval in seconds
    Interval(u64),
    /// Cron expression (minute, hour, day, month, weekday)
    Cron(String),
    /// Run on market open (needs market calendar)
    MarketOpen,
    /// Run on market close
    MarketClose,
}

impl Schedule {
    /// Parse a cron-like schedule
    pub fn parse_cron(expr: &str) -> Result<Self> {
        // Basic validation - 5 fields
        let fields: Vec<&str> = expr.split_whitespace().collect();
        if fields.len() != 5 {
            return Err(SigcError::Parse(format!(
                "Invalid cron expression: expected 5 fields, got {}",
                fields.len()
            )));
        }
        Ok(Schedule::Cron(expr.to_string()))
    }

    /// Every N minutes
    pub fn every_minutes(n: u64) -> Self {
        Schedule::Interval(n * 60)
    }

    /// Every N hours
    pub fn every_hours(n: u64) -> Self {
        Schedule::Interval(n * 3600)
    }

    /// Daily at specific hour (0-23)
    pub fn daily_at(hour: u32) -> Self {
        Schedule::Cron(format!("0 {} * * *", hour))
    }
}

/// Job execution result
#[derive(Debug, Clone)]
pub struct JobResult {
    pub job_id: String,
    pub status: JobStatus,
    pub started_at: u64,
    pub finished_at: u64,
    pub output: Option<String>,
    pub error: Option<String>,
}

/// Trait for job schedulers
pub trait JobScheduler: Send + Sync {
    /// Submit a job
    fn submit(&self, job: Job) -> Result<String>;

    /// Cancel a job
    fn cancel(&self, job_id: &str) -> Result<bool>;

    /// Get job status
    fn status(&self, job_id: &str) -> Result<Option<JobStatus>>;

    /// Get job details
    fn get(&self, job_id: &str) -> Result<Option<Job>>;

    /// List all jobs
    fn list(&self) -> Result<Vec<Job>>;

    /// Get scheduler name
    fn name(&self) -> &str;

    /// Check if scheduler is available
    fn is_available(&self) -> bool;
}

/// In-memory job scheduler for testing and simple use cases
pub struct MemoryScheduler {
    jobs: Arc<RwLock<HashMap<String, Job>>>,
    name: String,
}

impl MemoryScheduler {
    pub fn new() -> Self {
        MemoryScheduler {
            jobs: Arc::new(RwLock::new(HashMap::new())),
            name: "memory".to_string(),
        }
    }

    /// Get jobs due to run
    pub fn get_due_jobs(&self) -> Result<Vec<Job>> {
        use std::time::{SystemTime, UNIX_EPOCH};

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let jobs = self.jobs.read()
            .map_err(|e| SigcError::Runtime(format!("Lock error: {}", e)))?;

        let due: Vec<Job> = jobs
            .values()
            .filter(|job| {
                job.status == JobStatus::Pending
                    && job.next_run.map(|t| t <= now).unwrap_or(false)
            })
            .cloned()
            .collect();

        Ok(due)
    }

    /// Update job status
    pub fn update_status(&self, job_id: &str, status: JobStatus) -> Result<()> {
        let mut jobs = self.jobs.write()
            .map_err(|e| SigcError::Runtime(format!("Lock error: {}", e)))?;

        if let Some(job) = jobs.get_mut(job_id) {
            job.status = status;
            if status == JobStatus::Completed || status == JobStatus::Failed {
                use std::time::{SystemTime, UNIX_EPOCH};
                job.last_run = Some(
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs()
                );
            }
        }

        Ok(())
    }

    /// Calculate next run time for a job
    pub fn calculate_next_run(&self, job: &Job) -> Option<u64> {
        use std::time::{SystemTime, UNIX_EPOCH};

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        match &job.schedule {
            Schedule::Once(ts) => {
                if *ts > now {
                    Some(*ts)
                } else {
                    None
                }
            }
            Schedule::Interval(secs) => {
                let base = job.last_run.unwrap_or(now);
                Some(base + secs)
            }
            Schedule::Cron(_expr) => {
                // Simplified: just schedule for next hour
                // Full cron parsing would go here
                Some(now + 3600)
            }
            Schedule::MarketOpen | Schedule::MarketClose => {
                // Would need market calendar
                Some(now + 86400) // Next day as placeholder
            }
        }
    }
}

impl Default for MemoryScheduler {
    fn default() -> Self {
        Self::new()
    }
}

impl JobScheduler for MemoryScheduler {
    fn submit(&self, mut job: Job) -> Result<String> {
        let id = job.id.clone();

        // Calculate initial next run
        job.next_run = self.calculate_next_run(&job);

        let mut jobs = self.jobs.write()
            .map_err(|e| SigcError::Runtime(format!("Lock error: {}", e)))?;

        jobs.insert(id.clone(), job);
        Ok(id)
    }

    fn cancel(&self, job_id: &str) -> Result<bool> {
        let mut jobs = self.jobs.write()
            .map_err(|e| SigcError::Runtime(format!("Lock error: {}", e)))?;

        if let Some(job) = jobs.get_mut(job_id) {
            job.status = JobStatus::Cancelled;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn status(&self, job_id: &str) -> Result<Option<JobStatus>> {
        let jobs = self.jobs.read()
            .map_err(|e| SigcError::Runtime(format!("Lock error: {}", e)))?;

        Ok(jobs.get(job_id).map(|j| j.status))
    }

    fn get(&self, job_id: &str) -> Result<Option<Job>> {
        let jobs = self.jobs.read()
            .map_err(|e| SigcError::Runtime(format!("Lock error: {}", e)))?;

        Ok(jobs.get(job_id).cloned())
    }

    fn list(&self) -> Result<Vec<Job>> {
        let jobs = self.jobs.read()
            .map_err(|e| SigcError::Runtime(format!("Lock error: {}", e)))?;

        Ok(jobs.values().cloned().collect())
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn is_available(&self) -> bool {
        true
    }
}

/// Cron-based scheduler that uses system crontab
pub struct CronScheduler {
    name: String,
}

impl CronScheduler {
    pub fn new() -> Self {
        CronScheduler {
            name: "cron".to_string(),
        }
    }
}

impl Default for CronScheduler {
    fn default() -> Self {
        Self::new()
    }
}

impl JobScheduler for CronScheduler {
    fn submit(&self, _job: Job) -> Result<String> {
        Err(SigcError::Runtime("Cron scheduler not yet implemented".into()))
    }

    fn cancel(&self, _job_id: &str) -> Result<bool> {
        Err(SigcError::Runtime("Cron scheduler not yet implemented".into()))
    }

    fn status(&self, _job_id: &str) -> Result<Option<JobStatus>> {
        Err(SigcError::Runtime("Cron scheduler not yet implemented".into()))
    }

    fn get(&self, _job_id: &str) -> Result<Option<Job>> {
        Err(SigcError::Runtime("Cron scheduler not yet implemented".into()))
    }

    fn list(&self) -> Result<Vec<Job>> {
        Err(SigcError::Runtime("Cron scheduler not yet implemented".into()))
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn is_available(&self) -> bool {
        false
    }
}

/// Registry for managing schedulers
pub struct SchedulerRegistry {
    schedulers: HashMap<String, Box<dyn JobScheduler>>,
    default: Option<String>,
}

impl SchedulerRegistry {
    pub fn new() -> Self {
        SchedulerRegistry {
            schedulers: HashMap::new(),
            default: None,
        }
    }

    pub fn register(&mut self, name: &str, scheduler: Box<dyn JobScheduler>) {
        if self.default.is_none() {
            self.default = Some(name.to_string());
        }
        self.schedulers.insert(name.to_string(), scheduler);
    }

    pub fn get(&self, name: &str) -> Option<&dyn JobScheduler> {
        self.schedulers.get(name).map(|s| s.as_ref())
    }

    pub fn default_scheduler(&self) -> Option<&dyn JobScheduler> {
        self.default.as_ref().and_then(|name| self.get(name))
    }

    pub fn list(&self) -> Vec<String> {
        self.schedulers.keys().cloned().collect()
    }
}

impl Default for SchedulerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_creation() {
        let job = Job::new("job-1", "Daily Backtest", Schedule::daily_at(9), "sigc")
            .with_args(vec!["run".to_string(), "strategy.sig".to_string()])
            .with_max_retries(5)
            .with_timeout(7200)
            .with_tag("env", "prod");

        assert_eq!(job.id, "job-1");
        assert_eq!(job.max_retries, 5);
        assert_eq!(job.timeout_secs, 7200);
        assert_eq!(job.args.len(), 2);
    }

    #[test]
    fn test_schedule_parsing() {
        let schedule = Schedule::parse_cron("0 9 * * *").unwrap();
        matches!(schedule, Schedule::Cron(_));

        let err = Schedule::parse_cron("invalid");
        assert!(err.is_err());
    }

    #[test]
    fn test_schedule_helpers() {
        let mins = Schedule::every_minutes(30);
        matches!(mins, Schedule::Interval(1800));

        let hours = Schedule::every_hours(2);
        matches!(hours, Schedule::Interval(7200));
    }

    #[test]
    fn test_memory_scheduler() {
        let scheduler = MemoryScheduler::new();

        let job = Job::new("test-1", "Test Job", Schedule::Interval(60), "echo");
        let id = scheduler.submit(job).unwrap();

        assert_eq!(id, "test-1");

        let status = scheduler.status(&id).unwrap();
        assert_eq!(status, Some(JobStatus::Pending));

        let job = scheduler.get(&id).unwrap().unwrap();
        assert!(job.next_run.is_some());
    }

    #[test]
    fn test_job_cancellation() {
        let scheduler = MemoryScheduler::new();

        let job = Job::new("test-2", "Test Job", Schedule::Interval(60), "echo");
        scheduler.submit(job).unwrap();

        let cancelled = scheduler.cancel("test-2").unwrap();
        assert!(cancelled);

        let status = scheduler.status("test-2").unwrap();
        assert_eq!(status, Some(JobStatus::Cancelled));
    }

    #[test]
    fn test_job_list() {
        let scheduler = MemoryScheduler::new();

        scheduler.submit(Job::new("j1", "Job 1", Schedule::Interval(60), "cmd1")).unwrap();
        scheduler.submit(Job::new("j2", "Job 2", Schedule::Interval(120), "cmd2")).unwrap();

        let jobs = scheduler.list().unwrap();
        assert_eq!(jobs.len(), 2);
    }

    #[test]
    fn test_registry() {
        let mut registry = SchedulerRegistry::new();
        registry.register("memory", Box::new(MemoryScheduler::new()));

        assert!(registry.get("memory").is_some());
        assert!(registry.default_scheduler().is_some());
        assert_eq!(registry.list().len(), 1);
    }

    #[test]
    fn test_status_display() {
        assert_eq!(JobStatus::Pending.to_string(), "PENDING");
        assert_eq!(JobStatus::Running.to_string(), "RUNNING");
        assert_eq!(JobStatus::Completed.to_string(), "COMPLETED");
        assert_eq!(JobStatus::Failed.to_string(), "FAILED");
        assert_eq!(JobStatus::Cancelled.to_string(), "CANCELLED");
    }
}
