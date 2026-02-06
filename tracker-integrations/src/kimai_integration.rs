// SPDX-License-Identifier: MPL-2.0

use std::error::Error;

use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use reqwest::{Client, Url, header::CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
    integration::TrackerIntegration,
    models::{Activity, ApiId, Project, ProjectContext, Tag, TimeEntry, TimeEntryContext},
};

pub struct Unauthenticated;
#[derive(Clone)]
pub struct Authenticated {
    api_key: String,
    base_url: Url,
}

#[derive(Clone)]
pub struct KimaiClient<Auth> {
    client: Client,
    auth_state: Auth,
}

impl KimaiClient<Unauthenticated> {
    pub fn new() -> Self {
        tracing::info!("Creating new Kimai Client.");
        Self {
            client: Client::new(),
            auth_state: Unauthenticated,
        }
    }

    pub fn authenticate(self, api_key: String, base_url: &str) -> KimaiClient<Authenticated> {
        let base_url = Url::parse(base_url).expect("Invalid URL input."); // TODO
        KimaiClient {
            client: self.client,
            auth_state: Authenticated { api_key, base_url },
        }
    }
}

// Entity

#[derive(Debug, Serialize, Deserialize, Clone)]
struct KimaiProject {
    id: i64,
    name: String,
    color: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct KimaiTag {
    id: i64,
    name: String,
    color: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct KimaiTimeEntry {
    id: i64,
    activity: i64,
    project: i64,
    billable: bool,
    description: Option<String>,
    begin: DateTime<Utc>, // !TODO Research what implications that using UTC will have
    end: Option<DateTime<Utc>>, // !TODO Research what implications that using UTC will have
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct KimaiActivity {
    id: i64,
    name: String,
    project: i64,
    color: Option<String>,
}

// Expanded

#[derive(Debug, Serialize, Deserialize, Clone)]
struct KimaiTimeEntryExpanded {
    id: i64,
    activity: KimaiActivityExpanded,
    project: KimaiProject,
    billable: bool,
    description: Option<String>,
    begin: DateTime<Utc>,
    end: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct KimaiActivityExpanded {
    id: i64,
    name: String,
    project: KimaiProject,
    color: Option<String>,
}

impl From<KimaiProject> for Project {
    fn from(raw: KimaiProject) -> Self {
        Self {
            id: ApiId::Int(raw.id),
            name: raw.name,
            modified_at: DateTime::default(), // TODO
            color: raw.color.unwrap_or("ffffff".to_string()),
            context: ProjectContext::Kimai,
        }
    }
}

impl From<KimaiActivity> for Activity {
    fn from(raw: KimaiActivity) -> Self {
        Self {
            id: ApiId::Int(raw.id),
            name: raw.name,
            project_id: ApiId::Int(raw.project),
        }
    }
}

impl From<KimaiTag> for Tag {
    fn from(raw: KimaiTag) -> Self {
        Self {
            id: ApiId::Int(raw.id),
            name: raw.name,
            modified_at: DateTime::default(),
            workspace_id: todo!(), // TODO
        }
    }
}

impl From<KimaiTimeEntry> for TimeEntry {
    fn from(raw: KimaiTimeEntry) -> Self {
        Self {
            id: ApiId::Int(raw.id),
            billable: raw.billable,
            description: raw.description,
            duration: Duration::zero(), // TODO
            start_time: raw.begin,
            stop_time: raw.end,
            context: TimeEntryContext::Kimai {
                activity_id: ApiId::Int(raw.activity),
                project_id: ApiId::Int(raw.id),
            },
        }
    }
}

#[async_trait]
impl TrackerIntegration for KimaiClient<Authenticated> {
    async fn get_current_time_entry(&self) -> Result<Option<TimeEntry>, reqwest::Error> {
        tracing::info!("Getting current time entry.");
        let resp: Vec<KimaiTimeEntryExpanded> = self
            .client
            .get(
                self.auth_state
                    .base_url
                    .join("api/timesheets/active")
                    .unwrap(),
            )
            .bearer_auth(&self.auth_state.api_key)
            .header(CONTENT_TYPE, "application/json")
            .send()
            .await
            .expect("1")
            .json()
            .await
            .expect("2");
        match resp.first() {
            Some(active_entry) => {
                let entry = KimaiTimeEntry {
                    id: active_entry.id,
                    activity: active_entry.activity.id,
                    project: active_entry.project.id,
                    billable: active_entry.billable,
                    description: active_entry.description.clone(),
                    begin: active_entry.begin,
                    end: active_entry.end,
                };
                return Ok(Some(entry.into()));
            }
            None => Ok(None),
        }
    }

    async fn get_project_activities(
        &self,
        project_id: ApiId,
    ) -> Result<Vec<Activity>, reqwest::Error> {
        tracing::info!("Getting project activities.");
        let body = json!({
            "project": project_id,
        });

        let resp: Vec<KimaiActivity> = self
            .client
            .get(self.auth_state.base_url.join("api/activities").unwrap())
            .bearer_auth(&self.auth_state.api_key)
            .header(CONTENT_TYPE, "application/json")
            .json(&body)
            .send()
            .await?
            .json()
            .await?;
        Ok(resp.into_iter().map(Into::into).collect())
    }

    async fn get_projects(
        &self,
        _project_context: ProjectContext,
    ) -> Result<Vec<Project>, Box<dyn Error + Send + Sync + 'static>> {
        tracing::info!("Getting projects.");
        let resp: Vec<KimaiProject> = self
            .client
            .get(self.auth_state.base_url.join("api/projects").unwrap())
            .bearer_auth(&self.auth_state.api_key)
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
        let _ = time_entry_context;
        let _resp = self
            .client
            .patch(
                self.auth_state
                    .base_url
                    .join(&format!("/api/timesheets/{}/stop", time_entry_id))
                    .unwrap(),
            )
            .bearer_auth(&self.auth_state.api_key)
            .header(CONTENT_TYPE, "application/json")
            .send()
            .await?;
        Ok(())
    }

    async fn start_new_time_entry(
        &self,
        time_entry_context: TimeEntryContext,
        description: Option<String>,
    ) -> Result<TimeEntry, Box<dyn Error + Send + Sync + 'static>> {
        tracing::info!("Starting new time entry.");
        if let TimeEntryContext::Kimai {
            activity_id,
            project_id,
        } = time_entry_context
        {
            let body = json!({
                "activity": activity_id,
                "project": project_id,
                "description": description,
                "begin": Utc::now().to_rfc3339(),
            });

            let resp: KimaiTimeEntry = self
                .client
                .post(self.auth_state.base_url.join("/api/timesheets").unwrap())
                .bearer_auth(&self.auth_state.api_key)
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

    async fn get_user_workspaces(&self) -> Result<Vec<crate::models::Workspace>, reqwest::Error> {
        todo!()
    }
}
