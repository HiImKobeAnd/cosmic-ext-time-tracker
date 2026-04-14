use std::io::{self, BufRead};

use chrono::{DateTime, Duration};
use serde::Serialize;
use serde_json::{Error, json};
use tracker_integrations::{
    authentication::{get_api_key, get_integration_url},
    integration::TrackerIntegration,
    kimai_integration::{self, KimaiClient},
    models::{ApiId, Integration, TimeEntry, TimeEntryContext},
};

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct StartEntryData {
    context: TimeEntryContext,
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
    #[serde(rename = "get_all_activities")]
    GetAllActivities(()),
    #[serde(rename = "get_all_projects")]
    GetAllProjets(()),
}

impl BackendMessage {
    fn as_name(&self) -> &'static str {
        match self {
            // BackendMessage::IntegrationChanged(_) => "integration_changed",
            BackendMessage::GetCurrentEntry(_) => "get_current_time_entry",
            BackendMessage::StopEntry(_) => "stop_current_time_entry",
            BackendMessage::StartEntry(_) => "start_time_entry",
            BackendMessage::GetAllActivities(_) => "get_all_activities",
            BackendMessage::GetAllProjets(_) => "get_all_projects",
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

    for line in stdin.lock().lines() {
        if let Ok(message) = line {
            println!("{:#?}", &message);
            let message: Result<BackendMessage, Error> = serde_json::from_str(&message);
            match message {
                Ok(message) => parse_message(&client, message).await,
                Err(e) => println!("Failed: {:#?}", e),
            }
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
        BackendMessage::StopEntry(entry) => {
            client.stop_time_entry(entry.context, entry.id).await;
            send_to_stdout("stop_current_time_entry", "Success");
        }
        BackendMessage::StartEntry(start_entry_data) => {
            let result = client
                .start_new_time_entry(start_entry_data.context, start_entry_data.description)
                .await;
            if let Ok(result) = result {
                send_to_stdout("start_time_entry", result);
            }
        }
        BackendMessage::GetAllActivities(_) => {
            let result = client.get_all_activities().await;
            if let Ok(result) = result {
                send_to_stdout(message.as_name(), result);
            }
        }
        BackendMessage::GetAllProjets(_) => {
            let result = client.get_all_projects().await;
            if let Ok(result) = result {
                send_to_stdout(message.as_name(), result);
            }
        }
        _ => send_to_stdout("temp", "Not a valid message"),
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
