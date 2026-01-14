// SPDX-License-Identifier: MPL-2.0

use keyring::Entry;

pub fn set_api_key(key: String) -> Result<(), keyring::Error> {
    let entry = Entry::new("cosmic-ext-time-tracker", "toggl-api-key")?;
    entry.set_password(&key)?;
    tracing::info!("API key set.");
    Ok(())
}

pub fn get_api_key() -> Result<String, keyring::Error> {
    let entry = Entry::new("cosmic-ext-time-tracker", "toggl-api-key")?;
    let key = entry.get_password()?;
    Ok(key)
}
