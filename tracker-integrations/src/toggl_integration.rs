// SPDX-License-Identifier: MPL-2.0

use std::error::Error;

use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use reqwest::{Client, header::CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
    integration::TrackerIntegration,
    models::{ApiId, Project, ProjectContext, Tag, TimeEntry, TimeEntryContext, Workspace},
};

#[derive(Clone)]
pub struct TogglClient {
    client: Client,
    api_key: String,
}

impl TogglClient {
    pub async fn authenticate(
        api_key: String,
    ) -> Result<TogglClient, Box<dyn Error + Send + Sync + 'static>> {
        let integration = TogglClient {
            client: Client::new(),
            api_key,
        };
        match integration.validate_authentication().await {
            true => return Ok(integration),
            false => return Err("Test".into()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TogglProject {
    id: i64,
    name: String,
    workspace_id: i64,
    color: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TogglWorkspace {
    id: i64,
    name: String,
    active_project_count: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TogglTag {
    id: i64,
    name: String,
    workspace_id: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TogglTimeEntry {
    id: i64,
    workspace_id: i64,
    project_id: Option<i64>,
    billable: bool,
    description: Option<String>,
    // duration: Duration,
    start: DateTime<Utc>, // !TODO Research what implications that using UTC will have
    stop: Option<DateTime<Utc>>, // !TODO Research what implications that using UTC will have
}

impl From<TogglProject> for Project {
    fn from(raw: TogglProject) -> Self {
        Self {
            id: ApiId::Int(raw.id),
            name: raw.name,
            modified_at: DateTime::default(), // TODO
            color: raw.color,
            context: ProjectContext::Toggl {
                workspace_id: ApiId::Int(raw.workspace_id),
            },
        }
    }
}

impl From<TogglWorkspace> for Workspace {
    fn from(raw: TogglWorkspace) -> Self {
        Self {
            id: ApiId::Int(raw.id),
            name: raw.name,
            modified_at: DateTime::default(), // TODO
            active_project_count: raw.active_project_count,
        }
    }
}

impl From<TogglTag> for Tag {
    fn from(raw: TogglTag) -> Self {
        Self {
            id: ApiId::Int(raw.id),
            name: raw.name,
            modified_at: DateTime::default(), // TODO
            workspace_id: ApiId::Int(raw.workspace_id),
        }
    }
}

impl From<TogglTimeEntry> for TimeEntry {
    fn from(raw: TogglTimeEntry) -> Self {
        Self {
            id: ApiId::Int(raw.id),
            billable: raw.billable,
            description: raw.description,
            duration: Duration::zero(), // TODO
            start_time: raw.start,
            stop_time: raw.stop,
            // tags: todo!(),
            context: TimeEntryContext::Toggl {
                workspace_id: ApiId::Int(raw.workspace_id),
                project_id: raw.project_id.map(ApiId::Int),
            },
        }
    }
}

#[async_trait]
impl TrackerIntegration for TogglClient {
    async fn validate_authentication(&self) -> bool {
        tracing::info!("Checking authentication of Toggl Track");
        let resp = self
            .client
            .get("https://api.track.toggl.com/api/v9/me")
            .bearer_auth(&self.api_key)
            .header(CONTENT_TYPE, "application/json")
            .send()
            .await
            .expect("1");
        resp.status().is_success()
    }
    async fn get_current_time_entry(&self) -> Result<Option<TimeEntry>, reqwest::Error> {
        tracing::info!("Getting current time entry.");
        let resp: Option<TogglTimeEntry> = self
            .client
            .get("https://api.track.toggl.com/api/v9/me/time_entries/current")
            .basic_auth(&self.api_key, Some("api_token"))
            .header(CONTENT_TYPE, "application/json")
            .send()
            .await?
            .json()
            .await?;
        match resp {
            Some(entry) => Ok(Some(entry.into())),
            None => Ok(None),
        }
    }

    async fn get_user_workspaces(&self) -> Result<Vec<Workspace>, reqwest::Error> {
        tracing::info!("Getting user workspaces.");
        let resp: Vec<TogglWorkspace> = self
            .client
            .get("https://api.track.toggl.com/api/v9/me/workspaces")
            .basic_auth(&self.api_key, Some("api_token"))
            .header(CONTENT_TYPE, "application/json")
            .send()
            .await?
            .json()
            .await?;
        Ok(resp.into_iter().map(Into::into).collect())
    }

    async fn stop_time_entry(
        &self,
        time_entry_context: TimeEntryContext,
        time_entry_id: ApiId,
    ) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
        tracing::info!("Stopping time entry.");
        if let TimeEntryContext::Toggl {
            workspace_id,
            project_id: _,
        } = time_entry_context
        {
            let _resp = self
                .client
                .patch(format!(
                    "https://api.track.toggl.com/api/v9/workspaces/{}/time_entries/{}/stop",
                    workspace_id, time_entry_id
                ))
                .basic_auth(&self.api_key, Some("api_token"))
                .header(CONTENT_TYPE, "application/json")
                .send()
                .await?;
            Ok(())
        } else {
            Err("Test".into())
        }
    }

    async fn start_new_time_entry(
        &self,
        time_entry_context: TimeEntryContext,
        description: Option<String>,
    ) -> Result<TimeEntry, Box<dyn Error + Send + Sync + 'static>> {
        if let TimeEntryContext::Toggl {
            workspace_id,
            project_id,
        } = time_entry_context
        {
            tracing::info!("Starting new time entry.");
            let body = json!({
                "workspace_id": workspace_id,
                "project_id": project_id,
                "description": description,
                "created_with": "cosmic-ext-time-tracker",
                "duration": -1,
                "start": Utc::now().to_rfc3339(),
            });

            let resp: TogglTimeEntry = self
                .client
                .post(format!(
                    "https://api.track.toggl.com/api/v9/workspaces/{}/time_entries",
                    workspace_id
                ))
                .basic_auth(&self.api_key, Some("api_token"))
                .header(CONTENT_TYPE, "application/json")
                .json(&body)
                .send()
                .await?
                .json()
                .await?;
            Ok(resp.into())
        } else {
            Err("Test".into())
        }
    }

    async fn update_running_time_entry(
        &self,
        time_entry: &TimeEntry,
    ) -> Result<(), reqwest::Error> {
        tracing::info!("Updaing running time entry.");
        todo!()
    }

    async fn get_project_activities(
        &self,
        project_id: ApiId,
    ) -> Result<Vec<crate::models::Activity>, reqwest::Error> {
        todo!()
    }

    async fn get_projects(
        &self,
        project_context: ProjectContext,
    ) -> Result<Vec<Project>, Box<dyn Error + Send + Sync + 'static>> {
        tracing::info!("Getting workspace projects.");
        if let ProjectContext::Toggl { workspace_id } = project_context {
            let resp: Vec<TogglProject> = self
                .client
                .get(format!(
                    "https://api.track.toggl.com/api/v9/workspaces/{workspace_id}/projects"
                ))
                .basic_auth(&self.api_key, Some("api_token"))
                .header(CONTENT_TYPE, "application/json")
                .send()
                .await?
                .json()
                .await?;
            Ok(resp.into_iter().map(Into::into).collect())
        } else {
            Err("Test".into())
        }
    }
}
