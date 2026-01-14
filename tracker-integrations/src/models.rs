// SPDX-License-Identifier: MPL-2.0

use core::fmt;

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

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
    pub workspace_id: ApiId,
    pub color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Default)]
pub struct Workspace {
    pub id: ApiId,
    pub name: String,
    pub modified_at: DateTime<Utc>,
    pub active_project_count: i64,
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
    pub source_api: String,
    pub id: ApiId,
    pub billable: bool,
    pub description: Option<String>,
    pub duration: Duration,
    pub start_time: DateTime<Utc>, // !TODO Research what implications that using UTC will have
    pub stop_time: Option<DateTime<Utc>>, // !TODO Research what implications that using UTC will have
    pub project_id: Option<ApiId>,
    pub workspace_id: ApiId,
    pub tag_ids: Vec<ApiId>,
}
