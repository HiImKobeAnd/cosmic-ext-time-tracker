// SPDX-License-Identifier: MPL-2.0

use cosmic::cosmic_config::{self, cosmic_config_derive::CosmicConfigEntry, CosmicConfigEntry};
use tracker_integrations::models::{Activity, Integration, Project, Tag, TimeEntry, Workspace};

pub const GLOBAL_STATE_VERSION: u64 = 1;

#[derive(Debug, Default, Clone, CosmicConfigEntry, Eq, PartialEq)]
#[version = 1]
pub struct TrackerConfig {
    demo: String,
}

#[derive(Debug, Default, Clone, CosmicConfigEntry, Eq, PartialEq)]
#[version = 1]
pub struct GlobalState {
    pub selected_tracker: Option<Integration>,
    pub running_time_entry: Option<TimeEntry>,
    pub selected_workspace: Option<Workspace>,
    pub selected_activity: Option<Activity>,
    pub selected_project: Option<Project>,
    pub current_description: Option<String>,
    pub workspaces: Vec<Workspace>,
    pub projects: Vec<Project>,
    pub activities: Vec<Activity>,
    pub tags: Vec<Tag>,
}
