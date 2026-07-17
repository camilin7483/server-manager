use chrono::Utc;
use cron::Schedule;
use sm_core::id::JobId;
use sm_core::traits::{Job, JobDefinition, JobState};
use std::collections::HashMap;
use std::str::FromStr;
use tracing::info;

pub struct TaskScheduler {
    jobs: HashMap<JobId, Job>,
}

impl TaskScheduler {
    pub fn new() -> Self {
        Self {
            jobs: HashMap::new(),
        }
    }

    pub fn schedule(&mut self, definition: JobDefinition) -> JobId {
        let id = definition.id.unwrap_or_else(JobId::new);
        let job = Job {
            definition,
            state: JobState::Scheduled,
        };
        self.jobs.insert(id, job);
        info!("Tarea programada: {} ({})", id, self.jobs[&id].definition.name);
        id
    }

    pub fn cancel(&mut self, id: JobId) -> bool {
        if let Some(job) = self.jobs.get_mut(&id) {
            job.state = JobState::Cancelled;
            info!("Tarea cancelada: {}", id);
            true
        } else {
            false
        }
    }

    pub fn list(&self) -> Vec<&Job> {
        self.jobs.values().collect()
    }

    pub fn get(&self, id: JobId) -> Option<&Job> {
        self.jobs.get(&id)
    }

    pub fn next_tick(&self, id: JobId) -> Option<chrono::DateTime<Utc>> {
        let job = self.jobs.get(&id)?;
        let schedule = Schedule::from_str(&job.definition.cron_expr).ok()?;
        schedule.upcoming(Utc).next()
    }

    pub fn ready_jobs(&self) -> Vec<JobId> {
        let now = Utc::now();
        self.jobs
            .iter()
            .filter(|(_, job)| {
                if !job.definition.enabled || job.state != JobState::Scheduled {
                    return false;
                }
                if let Ok(schedule) = Schedule::from_str(&job.definition.cron_expr) {
                    schedule.includes(now)
                } else {
                    false
                }
            })
            .map(|(id, _)| *id)
            .collect()
    }
}
