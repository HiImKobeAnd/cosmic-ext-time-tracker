mod models;
use keyring::{Entry, mock};
use reqwest::{Client, Method};

use crate::models::TimeEntry;

pub fn get_current_time_entry() -> Option<TimeEntry> {
    set_default_store(mock::Store::new().expect("Could not set default store"));
    let client = Client::new();
    let resp = client.request(
        Method::GET,
        "https://api.track.toggl.com/api/v9/me/time_entries/current",
    );
}

pub fn set_api_key(key: String) {
    let entry = Entry::new("cosmic-ext-time-tracker", "toggl-api-key");
    entry.set_secret()
}
