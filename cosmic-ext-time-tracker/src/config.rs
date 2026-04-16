// SPDX-License-Identifier: MPL-2.0

use cosmic::cosmic_config::{self, cosmic_config_derive::CosmicConfigEntry, CosmicConfigEntry};
use tracker_integrations::models::{Integration, Project, Scope, TimeEntry};

pub const GLOBAL_STATE_VERSION: u64 = 1;

#[derive(Debug, Default, Clone, CosmicConfigEntry, Eq, PartialEq)]
#[version = 1]
pub struct GlobalState {
    pub selected_integration: Option<Integration>,
    pub running_time_entry: Option<TimeEntry>,
    pub selected_scope: Option<Scope>,
    pub selected_project: Option<Project>,
    pub current_description: Option<String>,
    pub scopes: Vec<Scope>,
    pub projects: Vec<Project>,
}
