// SPDX-License-Identifier: MPL-2.0

extern crate serde_json;

mod authentication;
mod models;
mod toggl_integration;

pub use authentication::*;
pub use models::*;
use serde::{Deserialize, Serialize};
pub use toggl_integration::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Integration {
    TogglIntegration,
}
