use std::error::Error;

use async_trait::async_trait;

use crate::models::{
    Activity, ApiId, Project, ProjectContext, TimeEntry, TimeEntryContext, Workspace,
};

#[async_trait]
pub trait TrackerIntegration {
    async fn validate_authentication(&self) -> bool;
    async fn get_current_time_entry(&self) -> Result<Option<TimeEntry>, reqwest::Error>;
    async fn get_project_activities(
        &self,
        project_id: ApiId,
    ) -> Result<Vec<Activity>, reqwest::Error>;
    async fn get_projects(
        &self,
        project_context: ProjectContext,
    ) -> Result<Vec<Project>, Box<dyn Error + Send + Sync + 'static>>;
    async fn get_user_workspaces(&self) -> Result<Vec<Workspace>, reqwest::Error>;
    // async fn get_workspace_projects(
    //     &self,
    //     workspace_id: ApiId,
    // ) -> Result<Vec<Project>, reqwest::Error>;
    async fn stop_time_entry(
        &self,
        time_entry_context: TimeEntryContext,
        time_entry_id: ApiId,
    ) -> Result<(), Box<dyn Error + Send + Sync + 'static>>;
    async fn start_new_time_entry(
        &self,
        time_entry_context: TimeEntryContext,
        description: Option<String>,
    ) -> Result<TimeEntry, Box<dyn Error + Send + Sync + 'static>>;
    async fn update_running_time_entry(&self, time_entry: &TimeEntry)
    -> Result<(), reqwest::Error>;
}
