// SPDX-License-Identifier: MPL-2.0

use core::fmt;
use std::sync::Arc;

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    authentication::{get_api_key, get_integration_url},
    integration::TrackerIntegration,
    kimai_integration::KimaiClient,
    toggl_integration::TogglClient,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Integration {
    TogglIntegration,
    KimaiIntegration,
}

impl std::fmt::Display for Integration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::TogglIntegration => "Toggl Track",
            Self::KimaiIntegration => "Kimai",
        })
    }
}

impl Integration {
    pub async fn create_client(&self) -> Option<Arc<dyn TrackerIntegration>> {
        match self {
            Integration::TogglIntegration => {
                let api_key = get_api_key(self).ok()?;
                TogglClient::authenticate(api_key)
                    .await
                    .ok()
                    .map(|c| Arc::new(c) as Arc<dyn TrackerIntegration>)
            }
            Integration::KimaiIntegration => {
                let api_key = get_api_key(self).ok()?;
                let base_url = get_integration_url(self).ok()?;
                KimaiClient::authenticate(api_key, &base_url)
                    .await
                    .ok()
                    .map(|c| Arc::new(c) as Arc<dyn TrackerIntegration>)
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
#[serde(untagged)]
pub enum ApiId {
    Int(i64),
    String(String),
}

impl Default for ApiId {
    fn default() -> Self {
        Self::Int(0)
    }
}

impl fmt::Display for ApiId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiId::Int(id) => write!(f, "{}", id),
            ApiId::String(id) => write!(f, "{}", id),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Default)]
pub struct Project {
    pub id: ApiId,
    pub name: String,
    pub modified_at: DateTime<Utc>,
    pub color: String,
    pub context: ProjectContext,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Default)]
pub struct Workspace {
    pub id: ApiId,
    pub name: String,
    pub modified_at: DateTime<Utc>,
    pub active_project_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Default)]
pub struct Activity {
    pub id: ApiId,
    pub name: String,
    pub project_id: ApiId,
    pub color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Default)]
pub struct Tag {
    pub id: ApiId,
    pub name: String,
    pub modified_at: DateTime<Utc>,
    pub workspace_id: ApiId,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Default)]
pub struct TimeEntry {
    pub id: ApiId,
    pub billable: bool,
    pub description: Option<String>,
    pub duration: Duration,
    pub start_time: DateTime<Utc>, // !TODO Research what implications that using UTC will have
    pub stop_time: Option<DateTime<Utc>>, // !TODO Research what implications that using UTC will have
    // pub tags: Vec<String>,
    pub context: TimeEntryContext,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Default)]
pub struct TimeEntryUpdate {
    pub billable: bool,
    pub description: Option<String>,
    pub start_time: DateTime<Utc>, // !TODO Research what implications that using UTC will have
    pub stop_time: Option<DateTime<Utc>>, // !TODO Research what implications that using UTC will have
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub enum ProjectContext {
    Kimai,
    Toggl { workspace_id: ApiId },
}

impl Default for ProjectContext {
    fn default() -> Self {
        Self::Kimai
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub enum TimeEntryContext {
    Kimai {
        activity_id: ApiId,
        project_id: ApiId,
    },
    Toggl {
        workspace_id: ApiId,
        project_id: Option<ApiId>,
    },
}

impl Default for TimeEntryContext {
    fn default() -> Self {
        Self::Kimai {
            activity_id: ApiId::default(),
            project_id: ApiId::default(),
        }
    }
}

impl TimeEntryContext {
    pub fn get_activity<'a>(&self, activities: &'a [Activity]) -> Option<&'a Activity> {
        match &self {
            TimeEntryContext::Kimai { activity_id, .. } => {
                activities.iter().find(|a| a.id == *activity_id)
            }
            _ => None,
        }
    }
    pub fn get_project<'a>(&self, projects: &'a [Project]) -> Option<&'a Project> {
        match &self {
            TimeEntryContext::Kimai { project_id, .. } => {
                projects.iter().find(|p| p.id == *project_id)
            }
            TimeEntryContext::Toggl { project_id, .. } => project_id
                .as_ref()
                .and_then(|id| projects.iter().find(|p| p.id == *id)),
        }
    }
    pub fn get_workspace<'a>(&self, workspaces: &'a [Workspace]) -> Option<&'a Workspace> {
        match &self {
            TimeEntryContext::Toggl { workspace_id, .. } => {
                workspaces.iter().find(|w| w.id == *workspace_id)
            }
            _ => None,
        }
    }
}
