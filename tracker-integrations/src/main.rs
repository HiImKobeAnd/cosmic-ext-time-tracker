// SPDX-License-Identifier: MPL-2.0

use std::io::{self, Write, stdin, stdout};

use tracker_integrations::{
    authentication::{get_api_key, set_api_key},
    models::Integration,
};

// Note for manual testing
#[tokio::main]
async fn main() {
    ensure_api_key().expect("Could not ensure API key.");
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
