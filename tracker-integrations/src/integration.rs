use std::fmt::Debug;

use async_trait::async_trait;

use crate::{
    error::Error,
    models::{
        Activity, ApiId, Project, ProjectContext, TimeEntry, TimeEntryContext, TimeEntryUpdate,
        Workspace,
    },
};

#[async_trait]
pub trait TrackerIntegration: Debug + Send + Sync {
    async fn validate_authentication(&self) -> Result<bool, Error>;
    async fn get_current_time_entry(&self) -> Result<Option<TimeEntry>, Error>;
    async fn get_project_activities(&self, project_id: ApiId) -> Result<Vec<Activity>, Error>;
    async fn get_projects(&self, project_context: ProjectContext) -> Result<Vec<Project>, Error>;
    async fn get_user_workspaces(&self) -> Result<Vec<Workspace>, Error>;
    // async fn get_workspace_projects(
    //     &self,
    //     workspace_id: ApiId,
    // ) -> Result<Vec<Project>, reqwest::Error>;
    async fn stop_time_entry(
        &self,
        time_entry_context: TimeEntryContext,
        time_entry_id: ApiId,
    ) -> Result<(), Error>;
    async fn start_new_time_entry(
        &self,
        time_entry_context: TimeEntryContext,
        description: Option<String>,
    ) -> Result<TimeEntry, Error>;
    async fn update_time_entry(
        &self,
        time_entry: &TimeEntry,
        time_entry_update: &TimeEntryUpdate,
    ) -> Result<TimeEntry, Error>;
}
