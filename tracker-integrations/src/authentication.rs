// SPDX-License-Identifier: MPL-2.0

use keyring::Entry;

use crate::models::Integration;

pub fn set_api_key(tracker: &Integration, key: String) -> Result<(), keyring::Error> {
    let tracker_name = tracker.to_string().replace(" ", "-").to_lowercase();
    let entry = Entry::new("cosmic-ext-time-tracker", &tracker_name)?;
    entry.set_password(&key)?;
    tracing::info!("API key set.");
    Ok(())
}

pub fn get_api_key(tracker: &Integration) -> Result<String, keyring::Error> {
    let tracker_name = tracker.to_string().replace(" ", "-").to_lowercase();
    let entry = Entry::new("cosmic-ext-time-tracker", &tracker_name)?;
    let key = entry.get_password()?;
    Ok(key)
}
