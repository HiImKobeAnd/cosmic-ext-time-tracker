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
    tracing::info!("API key gotten.");
    Ok(key)
}

pub fn remove_api_key(tracker: &Integration) -> Result<(), keyring::Error> {
    let tracker_name = tracker.to_string().replace(" ", "-").to_lowercase() + "-api-key";
    let entry = Entry::new("cosmic-ext-time-tracker", &tracker_name)?;
    entry.delete_credential()?;
    tracing::info!("API key removed.");
    Ok(())
}

pub fn set_integration_url(tracker: &Integration, url: String) -> Result<(), keyring::Error> {
    let tracker_name = tracker.to_string().replace(" ", "-").to_lowercase() + "-url";
    let entry = Entry::new("cosmic-ext-time-tracker", &tracker_name)?;
    entry.set_password(&url)?;
    tracing::info!("Integration URL set.");
    Ok(())
}

pub fn get_integration_url(tracker: &Integration) -> Result<String, keyring::Error> {
    let tracker_name = tracker.to_string().replace(" ", "-").to_lowercase() + "-url";
    let entry = Entry::new("cosmic-ext-time-tracker", &tracker_name)?;
    let url = entry.get_password()?;
    tracing::info!("Integration URL gotten.");
    Ok(url)
}

pub fn remove_integration_url(tracker: &Integration) -> Result<(), keyring::Error> {
    let tracker_name = tracker.to_string().replace(" ", "-").to_lowercase() + "-url";
    let entry = Entry::new("cosmic-ext-time-tracker", &tracker_name)?;
    entry.delete_credential()?;
    tracing::info!("Integration URL removed.");
    Ok(())
}
