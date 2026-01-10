// SPDX-License-Identifier: MPL-2.0

use core::fmt;

use chrono::{DateTime, Duration, Local, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum ApiId {
    Int(i64),
    String(String),
}

impl fmt::Display for ApiId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiId::Int(id) => write!(f, "{}", id),
            ApiId::String(id) => write!(f, "{}", id),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Project {
    pub id: ApiId,
    pub name: String,
    pub modified_at: DateTime<Utc>,
    pub workspace_id: ApiId,
    pub color: String,
}

#[derive(Debug, Clone)]
pub struct Workspace {
    pub id: ApiId,
    pub name: String,
    pub modified_at: DateTime<Utc>,
    pub active_project_count: i64,
}

#[derive(Debug, Clone)]
pub struct Tag {
    pub id: ApiId,
    pub name: String,
    pub modified_at: DateTime<Utc>,
    pub workspace_id: ApiId,
}

#[derive(Debug, Clone)]
pub struct TimeEntry {
    pub source_api: String,
    pub id: ApiId,
    pub billable: bool,
    pub description: String,
    pub duration: Duration,
    pub start_time: DateTime<Utc>, // !TODO Research what implications that using UTC will have
    pub stop_time: Option<DateTime<Utc>>, // !TODO Research what implications that using UTC will have
    pub project_id: Option<ApiId>,
    pub workspace_id: ApiId,
    pub tag_ids: Vec<ApiId>,
}
