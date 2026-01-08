// SPDX-License-Identifier: MPL-2.0

mod authentication;
mod models;
mod toggl_integration;

use std::io::{self, Write, stdin, stdout};

use crate::{
    authentication::{get_api_key, set_api_key},
    models::Workspace,
    toggl_integration::TogglClient,
};

// Note for manual testing
#[tokio::main]
async fn main() {
    // tracing_subscriber::fmt::init();
    // let _ = tracing_log::LogTracer::init();

    // tracing::info!("Staring time tracker applet with version {VERSION}");

    ensure_api_key().expect("Could not ensure API key.");
    let client = TogglClient::new();
    let time_entry = client
        .get_current_time_entry()
        .await
        .expect("Failed to get current time entry.");
    dbg!(time_entry);
    let client = TogglClient::new();
    let workspaces = client
        .get_user_workspaces()
        .await
        .expect("Failed to get workspaces.");
    dbg!(&workspaces);
    let client = TogglClient::new();

    match workspaces.first() {
        Some(workspace) => {
            let projects = client
                .get_workspace_projects(workspace.id.clone())
                .await
                .expect("Failed to get workspace projects.");
            dbg!(projects);
        }
        None => tracing::warn!("No workspaces to check for projects."),
    }
}

fn ensure_api_key() -> io::Result<()> {
    dbg!("Ensure api key");
    let key = get_api_key();
    if key.is_ok() {
        return Ok(());
    }
    let mut buf = String::new();
    println!("Input key:");
    stdout().flush()?;
    stdin().read_line(&mut buf)?;
    let trimmed = buf.trim().to_string();
    set_api_key(trimmed.clone()).unwrap();
    Ok(())
}
