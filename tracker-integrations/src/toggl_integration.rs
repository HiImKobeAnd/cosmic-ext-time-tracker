// SPDX-License-Identifier: MPL-2.0

use chrono::{DateTime, Duration, Utc};
use reqwest::{Client, header::CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
    authentication::{self},
    models::{ApiId, Project, Tag, TimeEntry, Workspace},
};

#[derive(Clone)]
pub struct TogglClient {
    client: Client,
    api_key: String,
}

impl TogglClient {
    pub fn new() -> Self {
        tracing::info!("Creating new Toggl Client.");
        Self {
            client: Client::new(),
            api_key: authentication::get_api_key().unwrap(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TogglProject {
    id: i64,
    name: String,
    // modified_at: DateTime<Utc>,
    workspace_id: i64,
    color: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TogglWorkspace {
    id: i64,
    name: String,
    // modified_at: DateTime<Utc>,
    active_project_count: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TogglTag {
    id: i64,
    // modified_at: DateTime<Utc>,
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
                          // tag_ids: Option<Vec<i64>>,
}

impl From<TogglProject> for Project {
    fn from(raw: TogglProject) -> Self {
        Self {
            id: ApiId::Int(raw.id),
            name: raw.name,
            modified_at: DateTime::default(), // TODO
            workspace_id: ApiId::Int(raw.workspace_id),
            color: raw.color,
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
            source_api: "TogglTrack".to_string(),
            id: ApiId::Int(raw.id),
            billable: raw.billable,
            description: raw.description,
            duration: Duration::zero(), // TODO
            start_time: raw.start,
            stop_time: raw.stop,
            project_id: raw.project_id.map(ApiId::Int),
            workspace_id: ApiId::Int(raw.workspace_id),
            tag_ids: Vec::new(), // TODO
        }
    }
}

impl TogglClient {
    pub async fn get_current_time_entry(&self) -> Result<Option<TimeEntry>, reqwest::Error> {
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

    pub async fn get_user_workspaces(&self) -> Result<Vec<Workspace>, reqwest::Error> {
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

    pub async fn get_workspace_projects(
        &self,
        workspace_id: ApiId,
    ) -> Result<Vec<Project>, reqwest::Error> {
        tracing::info!("Getting workspace projects.");
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
    }

    pub async fn stop_time_entry(
        &self,
        workspace_id: ApiId,
        time_entry_id: ApiId,
    ) -> Result<(), reqwest::Error> {
        tracing::info!("Stopping running time entry.");
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
    }

    pub async fn start_new_time_entry(
        &self,
        workspace_id: ApiId,
        project_id: Option<ApiId>,
        description: Option<String>,
    ) -> Result<TimeEntry, reqwest::Error> {
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
    }

    pub async fn update_running_time_entry(time_entry: &TimeEntry) -> Result<(), reqwest::Error> {
        tracing::info!("Updaing running time entry.");
        todo!()
    }
}
