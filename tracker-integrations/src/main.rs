// SPDX-License-Identifier: MPL-2.0

use std::io::{self, Write, stdin, stdout};

use tracker_integrations::{
    authentication::{get_api_key, set_api_key},
    models::Integration,
};

// Note for manual testing
#[tokio::main]
async fn main() {
    // tracing_subscriber::fmt::init();
    // let _ = tracing_log::LogTracer::init();

    // tracing::info!("Staring time tracker applet with version {VERSION}");

    ensure_api_key().expect("Could not ensure API key.");
    // let workspaces = TogglClient::get_user_workspaces()
    //     .await
    //     .expect("Failed to get workspaces.");
    // dbg!(&workspaces);
    // match workspaces.first() {
    //     Some(workspace) => {
    //         // let projects = TogglClient::get_workspace_projects(workspace.id.clone())
    //         // .await
    //         // .expect("Failed to get workspace projects.");
    //         // dbg!(projects);
    //         dbg!("Running start new time entry");
    //         let new_time_entry = TogglClient::start_new_time_entry(workspace.id.clone())
    //             .await
    //             .expect("Failed to start ned time entry.");
    //         // dbg!(&new_time_entry);
    //     }
    //     None => tracing::warn!("No workspaces to check for projects."),
    // }
    // let time_entry = TogglClient::get_current_time_entry()
    // .await
    // .expect("Failed to get current time entry.");
    // dbg!(&time_entry);
    // if let Some(time_entry) = time_entry {
    // let _ = TogglClient::stop_time_entry(&time_entry);
    // }
}

fn ensure_api_key() -> io::Result<()> {
    dbg!("Ensure api key");
    let key = get_api_key(&Integration::TogglIntegration);
    if key.is_ok() {
        return Ok(());
    }
    let mut buf = String::new();
    println!("Input key:");
    stdout().flush()?;
    stdin().read_line(&mut buf)?;
    let trimmed = buf.trim().to_string();
    set_api_key(&Integration::TogglIntegration, trimmed.clone()).unwrap();
    Ok(())
}
