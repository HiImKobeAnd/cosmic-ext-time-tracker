// SPDX-License-Identifier: MPL-2.0

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use reqwest::{Client, header::CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
    error::Error,
    integration::TrackerIntegration,
    models::{ApiId, Project, Scope, TimeEntry, TimeEntryUpdate},
};

#[derive(Clone, Debug)]
pub struct TogglClient {
    client: Client,
    api_key: String,
}

impl TogglClient {
    pub async fn authenticate(api_key: String) -> Result<TogglClient, Error> {
        let integration = TogglClient {
            client: Client::new(),
            api_key,
        };
        if !integration.validate_authentication().await? {
            return Err(Error::NotAuthorized);
        }
        Ok(integration)
    }
}

// ### Entities ###

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
    start: DateTime<Utc>, // !TODO Research what implications that using UTC will have
    stop: Option<DateTime<Utc>>, // !TODO Research what implications that using UTC will have
}
impl From<TogglWorkspace> for Scope {
    fn from(raw: TogglWorkspace) -> Self {
        Self {
            id: ApiId::Int(raw.id),
            name: raw.name,
            color: "ffffff".to_string(), // Toggl workspaces do not have colors
        }
    }
}

impl From<TogglProject> for Project {
    fn from(raw: TogglProject) -> Self {
        Self {
            id: ApiId::Int(raw.id),
            scope_id: ApiId::Int(raw.workspace_id),
            name: raw.name,
            color: raw.color,
        }
    }
}

impl From<TogglTimeEntry> for TimeEntry {
    fn from(raw: TogglTimeEntry) -> Self {
        Self {
            id: ApiId::Int(raw.id),
            scope_id: Some(ApiId::Int(raw.workspace_id)),
            project_id: raw.project_id.map(ApiId::Int),
            billable: raw.billable,
            description: raw.description,
            start_time: raw.start,
            stop_time: raw.stop,
        }
    }
}

// impl From<TogglTag> for Tag {
//     fn from(raw: TogglTag) -> Self {
//         Self {
//             id: ApiId::Int(raw.id),
//             name: raw.name,
//             modified_at: DateTime::default(), // TODO
//             workspace_id: ApiId::Int(raw.workspace_id),
//         }
//     }
// }

#[async_trait]
impl TrackerIntegration for TogglClient {
    async fn validate_authentication(&self) -> Result<bool, Error> {
        tracing::info!("Checking authentication of Toggl Track");
        let resp = self
            .client
            .get("https://api.track.toggl.com/api/v9/me/logged")
            .basic_auth(&self.api_key, Some("api_token"))
            .header(CONTENT_TYPE, "application/json")
            .send()
            .await?;
        Ok(resp.status().is_success())
    }
    async fn get_current_time_entry(&self) -> Result<Option<TimeEntry>, Error> {
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

    async fn get_all_scopes(&self) -> Result<Vec<Scope>, Error> {
        tracing::info!("Getting all workspaces.");
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

    async fn get_all_projects(&self) -> Result<Vec<Project>, Error> {
        tracing::info!("Getting all projects.");
        let resp: Vec<TogglProject> = self
            .client
            .get("https://api.track.toggl.com/api/v9/me/projects")
            .basic_auth(&self.api_key, Some("api_token"))
            .header(CONTENT_TYPE, "application/json")
            .send()
            .await?
            .json()
            .await?;
        Ok(resp.into_iter().map(Into::into).collect())
    }

    async fn stop_time_entry(&self, time_entry: &TimeEntry) -> Result<(), Error> {
        tracing::info!("Stopping time entry.");
        let Some(workspace_id) = &time_entry.scope_id else {
            return Err(Error::MissingRequiredField("Workspace ID".to_string()));
        };

        let _resp = self
            .client
            .patch(format!(
                "https://api.track.toggl.com/api/v9/workspaces/{}/time_entries/{}/stop",
                workspace_id, time_entry.id
            ))
            .basic_auth(&self.api_key, Some("api_token"))
            .header(CONTENT_TYPE, "application/json")
            .send()
            .await?;
        Ok(())
    }

    async fn start_new_time_entry(
        &self,
        scope_id: ApiId,
        project_id: Option<ApiId>,
        description: Option<String>,
    ) -> Result<TimeEntry, Error> {
        tracing::info!("Starting new time entry.");
        let body = json!({
            "workspace_id": scope_id,
            "project_id": project_id,
            "description": description,
            "created_with": "cosmic-ext-time-tracker",
            "start": Utc::now().to_rfc3339(),
        });

        let resp: TogglTimeEntry = self
            .client
            .post(format!(
                "https://api.track.toggl.com/api/v9/workspaces/{}/time_entries",
                scope_id
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

    async fn update_time_entry(
        &self,
        time_entry: &TimeEntry,
        time_entry_update: &TimeEntryUpdate,
    ) -> Result<TimeEntry, Error> {
        tracing::info!("Updaing time entry {}.", time_entry.id);
        let Some(workspace_id) = &time_entry.scope_id else {
            return Err(Error::MissingRequiredField("Workspace ID".to_string()));
        };

        let body = json!({
            "start": time_entry_update.start_time ,
            "stop": time_entry_update.stop_time,
            "description": time_entry_update.description,
            "billable": time_entry_update.billable,
        });

        let endpoint = format!(
            "https://api.track.toggl.com/api/v9/workspaces/{}/time_entries/{}",
            workspace_id, time_entry.id
        );
        let resp: TogglTimeEntry = self
            .client
            .put(endpoint)
            .basic_auth(&self.api_key, Some("api_token"))
            .header(CONTENT_TYPE, "application/json")
            .json(&body)
            .send()
            .await?
            .json()
            .await?;
        Ok(resp.into())
    }
}
