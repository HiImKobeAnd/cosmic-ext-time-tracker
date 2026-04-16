// SPDX-License-Identifier: MPL-2.0

use core::fmt;
use std::sync::Arc;

use chrono::{DateTime, Utc};
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
    pub fn all() -> &'static [Self] {
        &[Self::KimaiIntegration, Self::TogglIntegration]
    }
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

// Main domain

// Kimai project & Toggl workspace
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Default)]
pub struct Scope {
    pub id: ApiId,
    pub name: String,
    pub color: String,
}

// Kimai activity & Toggl Project
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Default)]
pub struct Project {
    pub id: ApiId,
    pub scope_id: ApiId,
    pub name: String,
    pub color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Default)]
pub struct TimeEntry {
    pub id: ApiId,
    pub scope_id: Option<ApiId>,
    pub project_id: Option<ApiId>,
    pub billable: bool,
    pub description: Option<String>,
    pub start_time: DateTime<Utc>, // !TODO Research what implications that using UTC will have
    pub stop_time: Option<DateTime<Utc>>, // !TODO Research what implications that using UTC will have
}

// #[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Default)]
// pub struct Tag {
//     pub id: ApiId,
//     pub name: String,
//     pub modified_at: DateTime<Utc>,
//     pub workspace_id: ApiId,
// }

// Extra DTOs

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Default)]
pub struct TimeEntryUpdate {
    pub billable: bool,
    pub description: Option<String>,
    pub start_time: DateTime<Utc>, // !TODO Research what implications that using UTC will have
    pub stop_time: Option<DateTime<Utc>>, // !TODO Research what implications that using UTC will have
}
