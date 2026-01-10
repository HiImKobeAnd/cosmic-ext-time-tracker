// SPDX-License-Identifier: MPL-2.0

use chrono::{DateTime, Duration, Utc};
use reqwest::{Client, header::CONTENT_TYPE};
use serde::{Deserialize, Serialize};

use crate::{
    authentication::get_api_key,
    models::{ApiId, Project, Tag, TimeEntry, Workspace},
};

pub struct TogglClient;

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
    billable: bool,
    description: String,
    // duration: Duration,
    start: DateTime<Utc>, // !TODO Research what implications that using UTC will have
    stop: Option<DateTime<Utc>>, // !TODO Research what implications that using UTC will have
    project_id: Option<i64>,
    workspace_id: i64,
    tag_ids: Vec<i64>,
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
            tag_ids: raw.tag_ids.into_iter().map(ApiId::Int).collect(),
        }
    }
}

impl TogglClient {
    pub async fn get_current_time_entry() -> Result<TimeEntry, reqwest::Error> {
        tracing::info!("Running get current time entry.");
        let client = Client::new();
        let api_key = get_api_key().unwrap();
        let resp: TogglTimeEntry = client
            .get("https://api.track.toggl.com/api/v9/me/time_entries/current")
            .basic_auth(api_key, Some("api_token"))
            .header(CONTENT_TYPE, "application/json")
            .send()
            .await
            .expect("Failed to get response.")
            .json()
            .await
            .expect("Failed to convert to JSON.");
        Ok(resp.into())
    }

    pub async fn get_user_workspaces() -> Result<Vec<Workspace>, reqwest::Error> {
        tracing::info!("Running get user workspaces.");
        let client = Client::new();
        let api_key = get_api_key().unwrap();
        let resp: Vec<TogglWorkspace> = client
            .get("https://api.track.toggl.com/api/v9/me/workspaces")
            .basic_auth(api_key, Some("api_token"))
            .header(CONTENT_TYPE, "application/json")
            .send()
            .await
            .expect("Failed to get response.")
            .json()
            .await
            .expect("Failed to convert to JSON.");
        Ok(resp.into_iter().map(Into::into).collect())
    }

    pub async fn get_workspace_projects(
        workspace_id: ApiId,
    ) -> Result<Vec<Project>, reqwest::Error> {
        tracing::info!("Running get workspace projects.");
        let client = Client::new();
        let api_key = get_api_key().unwrap();
        let resp: Vec<TogglProject> = client
            .get(format!(
                "https://api.track.toggl.com/api/v9/workspaces/{workspace_id}/projects"
            ))
            .basic_auth(api_key, Some("api_token"))
            .header(CONTENT_TYPE, "application/json")
            .send()
            .await
            .expect("Failed to get response.")
            .json()
            .await
            .expect("Failed to convert to JSON.");
        Ok(resp.into_iter().map(Into::into).collect())
    }

    pub async fn stop_time_entry(time_entry: &TimeEntry) -> Result<(), reqwest::Error> {
        tracing::info!("Stopping running time entry.");
        let client = Client::new();
        let api_key = get_api_key().unwrap();
        let resp = client
            .patch(format!(
                "https://api.track.toggl.com/api/v9/workspaces/{}/time_entries/{}/stop",
                time_entry.workspace_id, time_entry.id
            ))
            .basic_auth(api_key, Some("api_token"))
            .header(CONTENT_TYPE, "application/json")
            .send()
            .await
            .expect("Failed to get response.");
        Ok(())
    }

    pub async fn start_new_time_entry(workspace: Workspace) -> Result<(), reqwest::Error> {
        tracing::info!("Stopping running time entry.");
        let client = Client::new();
        let api_key = get_api_key().unwrap();
        let resp = client
            .post(format!(
                "https://api.track.toggl.com/api/v9/workspaces/{}/time_entries",
                workspace.id
            ))
            .basic_auth(api_key, Some("api_token"))
            .header(CONTENT_TYPE, "application/json")
            .send()
            .await
            .expect("Failed to get response.");
        dbg!(resp);
        Ok(())
    }
}
