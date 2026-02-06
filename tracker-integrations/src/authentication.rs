// SPDX-License-Identifier: MPL-2.0

use keyring::Entry;

use crate::models::Integration;

pub fn set_api_key(tracker: &Integration, key: String) -> Result<(), keyring::Error> {
    let tracker_name = tracker.to_string().replace(" ", "-").to_lowercase() + "-api-key";
    let entry = Entry::new("cosmic-ext-time-tracker", &tracker_name)?;
    entry.set_password(&key)?;
    tracing::info!("API key set.");
    Ok(())
}

pub fn get_api_key(tracker: &Integration) -> Result<String, keyring::Error> {
    let tracker_name = tracker.to_string().replace(" ", "-").to_lowercase() + "-api-key";
    let entry = Entry::new("cosmic-ext-time-tracker", &tracker_name)?;
    let key = entry.get_password()?;
    Ok(key)
}

pub fn set_integration_url(tracker: &Integration, url: String) -> Result<(), keyring::Error> {
    let tracker_name = tracker.to_string().replace(" ", "-").to_lowercase() + "-url";
    let entry = Entry::new("cosmic-ext-time-tracker", &tracker_name)?;
    entry.set_password(&url)?;
    tracing::info!("API key set.");
    Ok(())
}

pub fn get_integration_url(tracker: &Integration) -> Result<String, keyring::Error> {
    let tracker_name = tracker.to_string().replace(" ", "-").to_lowercase() + "-url";
    let entry = Entry::new("cosmic-ext-time-tracker", &tracker_name)?;
    let url = entry.get_password()?;
    Ok(url)
}
