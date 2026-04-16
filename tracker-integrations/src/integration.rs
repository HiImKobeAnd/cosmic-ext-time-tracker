use std::fmt::Debug;

use async_trait::async_trait;

use crate::{
    error::Error,
    models::{Project, Scope, TimeEntry, TimeEntryUpdate},
};

#[async_trait]
pub trait TrackerIntegration: Debug + Send + Sync {
    async fn validate_authentication(&self) -> Result<bool, Error>;
    async fn get_current_time_entry(&self) -> Result<Option<TimeEntry>, Error>;
    async fn get_all_scopes(&self) -> Result<Vec<Scope>, Error>;
    async fn get_all_projects(&self) -> Result<Vec<Project>, Error>;
    async fn stop_time_entry(&self, time_entry: &TimeEntry) -> Result<(), Error>;
    async fn start_new_time_entry(
        &self,
        time_entry: &TimeEntry,
        description: Option<String>,
    ) -> Result<TimeEntry, Error>;
    async fn update_time_entry(
        &self,
        time_entry: &TimeEntry,
        time_entry_update: &TimeEntryUpdate,
    ) -> Result<TimeEntry, Error>;
}
