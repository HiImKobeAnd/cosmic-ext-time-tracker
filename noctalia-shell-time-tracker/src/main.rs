use std::io::{self, BufRead};

use serde::Serialize;
use serde_json::{Error, json};
use tracker_integrations::{
    authentication::{get_api_key, get_integration_url},
    integration::TrackerIntegration,
    kimai_integration::KimaiClient,
    models::{Integration, TimeEntry},
};

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct StartEntryData {
    scope_id: String,
    project_id: Option<String>,
    description: Option<String>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(tag = "message", content = "content")]
enum BackendMessage {
    // #[serde(rename = "integration_changed")]
    // IntegrationChanged(Integration),
    #[serde(rename = "get_current_time_entry")]
    GetCurrentEntry(()),
    #[serde(rename = "stop_current_time_entry")]
    StopEntry(TimeEntry),
    #[serde(rename = "start_time_entry")]
    StartEntry(StartEntryData),
    #[serde(rename = "get_all_integrations")]
    GetAllIntegrations(()),
    #[serde(rename = "get_all_scopes")]
    GetAllScopes(()),
    #[serde(rename = "get_all_projects")]
    GetAllProjects(()),
}

impl BackendMessage {
    fn as_name(&self) -> &'static str {
        match self {
            // BackendMessage::IntegrationChanged(_) => "integration_changed",
            BackendMessage::GetCurrentEntry(_) => "get_current_time_entry",
            BackendMessage::StopEntry(_) => "stop_current_time_entry",
            BackendMessage::StartEntry(_) => "start_time_entry",
            BackendMessage::GetAllIntegrations(_) => "get_all_integrations",
            BackendMessage::GetAllScopes(_) => "get_all_scopes",
            BackendMessage::GetAllProjects(_) => "get_all_projects",
        }
    }
}

#[tokio::main]
async fn main() {
    let stdin = io::stdin();

    let api_key = get_api_key(&Integration::KimaiIntegration).unwrap();
    let base_url = get_integration_url(&Integration::KimaiIntegration).expect("Could not get url");
    let client = KimaiClient::authenticate(api_key, &base_url)
        .await
        .expect("Could not get key");

    for message in stdin.lock().lines().flatten() {
        println!("{:#?}", &message);
        let message: Result<BackendMessage, Error> = serde_json::from_str(&message);
        match message {
            Ok(message) => parse_message(&client, message).await,
            Err(e) => println!("Failed: {:#?}", e),
        }
    }
}

async fn parse_message(client: &KimaiClient, message: BackendMessage) {
    match message {
        BackendMessage::GetCurrentEntry(_) => {
            if let Some(current_time_entry) = client.get_current_time_entry().await.unwrap() {
                send_to_stdout(message.as_name(), current_time_entry);
            };
        }
        BackendMessage::StopEntry(ref entry) => {
            let _ = client.stop_time_entry(entry).await;
            send_to_stdout(message.as_name(), "Success");
        }
        BackendMessage::StartEntry(ref start_entry_data) => {
            let result = client
                .start_new_time_entry(
                    start_entry_data.scope_id.clone(),
                    start_entry_data.project_id.clone(),
                    start_entry_data.description.clone(),
                )
                .await;
            if let Ok(result) = result {
                send_to_stdout(message.as_name(), result);
            }
        }
        BackendMessage::GetAllIntegrations(_) => {
            let result = Integration::all();
            send_to_stdout(message.as_name(), result);
        }

        BackendMessage::GetAllScopes(_) => {
            let result = client.get_all_scopes().await;
            if let Ok(result) = result {
                send_to_stdout(message.as_name(), result);
            }
        }
        BackendMessage::GetAllProjects(_) => {
            let result = client.get_all_projects().await;
            if let Ok(result) = result {
                send_to_stdout(message.as_name(), result);
            }
        }
    }
}

fn send_to_stdout(message: impl Serialize, content: impl Serialize) {
    println!(
        "{}",
        json!({
            "message": message,
            "content": content
        })
    );
}
