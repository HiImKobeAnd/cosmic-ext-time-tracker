// SPDX-License-Identifier: MPL-2.0

use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use reqwest::{Client, Url, header::CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
    error::Error,
    integration::TrackerIntegration,
    models::{
        Activity, ApiId, Project, ProjectContext, TimeEntry, TimeEntryContext, TimeEntryUpdate,
    },
};

#[derive(Clone, Debug)]
pub struct KimaiClient {
    client: Client,
    api_key: String,
    base_url: Url,
}

impl KimaiClient {
    pub async fn authenticate(api_key: String, base_url: &str) -> Result<KimaiClient, Error> {
        let base_url = Url::parse(base_url)?;
        let integration = KimaiClient {
            client: Client::new(),
            api_key,
            base_url,
        };
        if !integration.validate_authentication().await? {
            return Err(Error::NotAuthorized);
        }
        Ok(integration)
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
            color: raw.color.unwrap_or("ffffff".to_string()),
        }
    }
}

// impl From<KimaiTag> for Tag {
//     fn from(raw: KimaiTag) -> Self {
//         Self {
//             id: ApiId::Int(raw.id),
//             name: raw.name,
//             modified_at: DateTime::default(),
//             workspace_id: todo!(), // TODO
//         }
//     }
// }

impl From<KimaiTimeEntry> for TimeEntry {
    fn from(raw: KimaiTimeEntry) -> Self {
        Self {
            id: ApiId::Int(raw.id),
            billable: raw.billable,
            description: raw.description,
            duration: Duration::zero(), // TODO
            start_time: raw.begin,
            stop_time: raw.end,
            context: TimeEntryContext {
                activity_id: Some(ApiId::Int(raw.activity)),
                project_id: Some(ApiId::Int(raw.project)),
                workspace_id: None,
            },
        }
    }
}

#[async_trait]
impl TrackerIntegration for KimaiClient {
    async fn validate_authentication(&self) -> Result<bool, Error> {
        tracing::info!("Checking authentication of Kimai");
        let resp = self
            .client
            .get(self.base_url.join("api/users/me")?)
            .bearer_auth(&self.api_key)
            .header(CONTENT_TYPE, "application/json")
            .send()
            .await?;
        Ok(resp.status().is_success())
    }
    async fn get_current_time_entry(&self) -> Result<Option<TimeEntry>, Error> {
        tracing::info!("Getting current time entry.");
        let resp: Vec<KimaiTimeEntryExpanded> = self
            .client
            .get(self.base_url.join("api/timesheets/active")?)
            .bearer_auth(&self.api_key)
            .header(CONTENT_TYPE, "application/json")
            .send()
            .await?
            .json()
            .await?;
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
            None => return Ok(None),
        }
    }

    async fn get_project_activities(&self, project_id: ApiId) -> Result<Vec<Activity>, Error> {
        tracing::info!("Getting project activities.");
        let resp: Vec<KimaiActivity> = self
            .client
            .get(
                self.base_url
                    .join(&format!("api/activities?project={}", project_id))?,
            )
            .bearer_auth(&self.api_key)
            .header(CONTENT_TYPE, "application/json")
            .send()
            .await?
            .json()
            .await?;
        Ok(resp.into_iter().map(Into::into).collect())
    }

    async fn get_projects(&self, _project_context: ProjectContext) -> Result<Vec<Project>, Error> {
        tracing::info!("Getting projects.");
        let resp: Vec<KimaiProject> = self
            .client
            .get(self.base_url.join("api/projects")?)
            .bearer_auth(&self.api_key)
            .header(CONTENT_TYPE, "application/json")
            .send()
            .await?
            .json()
            .await?;
        Ok(resp.into_iter().map(Into::into).collect())
    }

    async fn get_user_workspaces(&self) -> Result<Vec<crate::models::Workspace>, Error> {
        todo!()
    }

    async fn stop_time_entry(
        &self,
        _time_entry_context: TimeEntryContext,
        time_entry_id: ApiId,
    ) -> Result<(), Error> {
        tracing::info!("Stopping time entry.");
        let _resp = self
            .client
            .patch(
                self.base_url
                    .join(&format!("/api/timesheets/{}/stop", time_entry_id))?,
            )
            .bearer_auth(&self.api_key)
            .header(CONTENT_TYPE, "application/json")
            .send()
            .await?;
        Ok(())
    }

    async fn start_new_time_entry(
        &self,
        time_entry_context: TimeEntryContext,
        description: Option<String>,
    ) -> Result<TimeEntry, Error> {
        tracing::info!("Starting new time entry.");
        let Some(activity_id) = time_entry_context.activity_id else {
            return Err(Error::MissingRequiredField("Activity ID".to_string()));
        };
        let Some(project_id) = time_entry_context.project_id else {
            return Err(Error::MissingRequiredField("Project ID".to_string()));
        };

        let body = json!({
            "activity": activity_id,
            "project": project_id,
            "description": description,
            "begin": Utc::now().to_rfc3339(),
        });

        let resp: KimaiTimeEntry = self
            .client
            .post(self.base_url.join("/api/timesheets")?)
            .bearer_auth(&self.api_key)
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

        let body = json!({
            "begin": time_entry_update.start_time ,
            "end": time_entry_update.stop_time,
            "description": time_entry_update.description,
            "billable": time_entry_update.billable,
        });

        let endpoint = format!("/api/timesheets/{}", time_entry.id);
        let resp: KimaiTimeEntry = self
            .client
            .patch(self.base_url.join(&endpoint)?)
            .bearer_auth(&self.api_key)
            .header(CONTENT_TYPE, "application/json")
            .json(&body)
            .send()
            .await?
            .json()
            .await?;
        Ok(resp.into())
    }
}
