// SPDX-License-Identifier: MPL-2.0

use cosmic::cosmic_config::{self, cosmic_config_derive::CosmicConfigEntry, CosmicConfigEntry};
use tracker_integrations::{Project, Tag, TimeEntry, Workspace};

pub const GLOBAL_STATE_VERSION: u64 = 1;

#[derive(Debug, Default, Clone, CosmicConfigEntry, Eq, PartialEq)]
#[version = 1]
pub struct TrackerConfig {
    demo: String,
}

#[derive(Debug, Default, Clone, CosmicConfigEntry, Eq, PartialEq)]
#[version = 1]
pub struct GlobalState {
    pub running_time_entry: Option<TimeEntry>,
    pub selected_workspace: Option<Workspace>,
    pub selected_project: Option<Project>,
    pub workspaces: Vec<Workspace>,
    pub projects_for_selected_workspace: Vec<Project>,
    pub tags: Vec<Tag>,
}
