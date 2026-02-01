// // SPDX-License-Identifier: MPL-2.0
//
// use chrono::{DateTime, Duration, Utc};
// use reqwest::{Client, header::CONTENT_TYPE};
// use serde::{Deserialize, Serialize};
// use serde_json::json;
//
// use crate::models::{ApiId, Project, Tag, TimeEntry, Workspace};
//
// pub struct Unauthenticated;
// #[derive(Clone)]
// pub struct Authenticated {
//     api_key: String,
// }
//
// #[derive(Clone)]
// pub struct KimaiClient<Auth> {
//     client: Client,
//     auth_state: Auth,
// }
//
// impl KimaiClient<Unauthenticated> {
//     pub fn new() -> Self {
//         tracing::info!("Creating new Kimai Client.");
//         Self {
//             client: Client::new(),
//             auth_state: Unauthenticated,
//         }
//     }
//
//     pub fn authenticate(self, api_key: String) -> KimaiClient<Authenticated> {
//         KimaiClient {
//             client: self.client,
//             auth_state: Authenticated { api_key },
//         }
//     }
// }
//
// #[derive(Debug, Serialize, Deserialize)]
// struct KimaiProject {
//     id: i64,
//     name: String,
//     color: String,
// }
//
// #[derive(Debug, Serialize, Deserialize)]
// struct KimaiTag {
//     id: i64,
//     name: String,
//     color: String,
// }
//
// #[derive(Debug, Serialize, Deserialize)]
// struct KimaiTimeEntry {
//     id: i64,
//     activity: Option<i64>,
//     project: Option<i64>,
//     billable: bool,
//     description: Option<String>,
//     begin: DateTime<Utc>, // !TODO Research what implications that using UTC will have
//     end: Option<DateTime<Utc>>, // !TODO Research what implications that using UTC will have
// }
//
// #[derive(Debug, Serialize, Deserialize)]
// struct KimaiActivity {
//     id: i64,
//     name: String,
//     project: i64,
//     color: String,
// }
//
// impl From<KimaiProject> for Project {
//     fn from(raw: KimaiProject) -> Self {
//         Self {
//             id: ApiId::Int(raw.id),
//             name: raw.name,
//             modified_at: DateTime::default(), // TODO
//             workspace_id: ApiId::Int(raw.workspace_id),
//             color: raw.color,
//         }
//     }
// }
//
// impl From<KimaiTag> for Tag {
//     fn from(raw: KimaiTag) -> Self {
//         Self {
//             id: ApiId::Int(raw.id),
//             name: raw.name,
//             modified_at: DateTime::default(), // TODO
//             workspace_id: ApiId::Int(raw.workspace_id),
//         }
//     }
// }
//
// impl From<KimaiTimeEntry> for TimeEntry {
//     fn from(raw: KimaiTimeEntry) -> Self {
//         Self {
//             source_api: "KimaiTrack".to_string(),
//             id: ApiId::Int(raw.id),
//             billable: raw.billable,
//             description: raw.description,
//             duration: Duration::zero(), // TODO
//             start_time: raw.start,
//             stop_time: raw.stop,
//             project_id: raw.project_id.map(ApiId::Int),
//             workspace_id: ApiId::Int(raw.workspace_id),
//             tag_ids: Vec::new(), // TODO
//         }
//     }
// }
//
// impl KimaiClient<Authenticated> {
//     pub async fn get_current_time_entry(&self) -> Result<Option<TimeEntry>, reqwest::Error> {
//         tracing::info!("Getting current time entry.");
//         let resp: Option<KimaiTimeEntry> = self
//             .client
//             .get("https://api.track.toggl.com/api/v9/me/time_entries/current")
//             .basic_auth(&self.auth_state.api_key, Some("api_token"))
//             .header(CONTENT_TYPE, "application/json")
//             .send()
//             .await?
//             .json()
//             .await?;
//         match resp {
//             Some(entry) => Ok(Some(entry.into())),
//             None => Ok(None),
//         }
//     }
//
//     pub async fn get_user_workspaces(&self) -> Result<Vec<Workspace>, reqwest::Error> {
//         tracing::info!("Getting user workspaces.");
//         let resp: Vec<KimaiWorkspace> = self
//             .client
//             .get("https://api.track.toggl.com/api/v9/me/workspaces")
//             .basic_auth(&self.auth_state.api_key, Some("api_token"))
//             .header(CONTENT_TYPE, "application/json")
//             .send()
//             .await?
//             .json()
//             .await?;
//         Ok(resp.into_iter().map(Into::into).collect())
//     }
//
//     pub async fn get_workspace_projects(
//         &self,
//         workspace_id: ApiId,
//     ) -> Result<Vec<Project>, reqwest::Error> {
//         tracing::info!("Getting workspace projects.");
//         let resp: Vec<KimaiProject> = self
//             .client
//             .get(format!(
//                 "https://api.track.toggl.com/api/v9/workspaces/{workspace_id}/projects"
//             ))
//             .basic_auth(&self.auth_state.api_key, Some("api_token"))
//             .header(CONTENT_TYPE, "application/json")
//             .send()
//             .await?
//             .json()
//             .await?;
//         Ok(resp.into_iter().map(Into::into).collect())
//     }
//
//     pub async fn stop_time_entry(
//         &self,
//         workspace_id: ApiId,
//         time_entry_id: ApiId,
//     ) -> Result<(), reqwest::Error> {
//         tracing::info!("Stopping running time entry.");
//         let _resp = self
//             .client
//             .patch(format!(
//                 "https://api.track.toggl.com/api/v9/workspaces/{}/time_entries/{}/stop",
//                 workspace_id, time_entry_id
//             ))
//             .basic_auth(&self.auth_state.api_key, Some("api_token"))
//             .header(CONTENT_TYPE, "application/json")
//             .send()
//             .await?;
//         Ok(())
//     }
//
//     pub async fn start_new_time_entry(
//         &self,
//         workspace_id: ApiId,
//         project_id: Option<ApiId>,
//         description: Option<String>,
//     ) -> Result<TimeEntry, reqwest::Error> {
//         tracing::info!("Starting new time entry.");
//         let body = json!({
//             "workspace_id": workspace_id,
//             "project_id": project_id,
//             "description": description,
//             "created_with": "cosmic-ext-time-tracker",
//             "duration": -1,
//             "start": Utc::now().to_rfc3339(),
//         });
//
//         let resp: KimaiTimeEntry = self
//             .client
//             .post(format!(
//                 "https://api.track.toggl.com/api/v9/workspaces/{}/time_entries",
//                 workspace_id
//             ))
//             .basic_auth(&self.auth_state.api_key, Some("api_token"))
//             .header(CONTENT_TYPE, "application/json")
//             .json(&body)
//             .send()
//             .await?
//             .json()
//             .await?;
//         Ok(resp.into())
//     }
//
//     pub async fn update_running_time_entry(time_entry: &TimeEntry) -> Result<(), reqwest::Error> {
//         tracing::info!("Updaing running time entry.");
//         todo!()
//     }
// }
