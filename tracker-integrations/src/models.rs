// SPDX-License-Identifier: MPL-2.0

use core::fmt;

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

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
