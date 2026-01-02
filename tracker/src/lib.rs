mod models;
use models;
use reqwest::Client;

pub fn get_current_time_entry() -> Option<TimeEntry> {
    let client = Client::new().
    let resp = reqwest::get("https://api.track.toggl.com/api/v9/me/time_entries/current")
}


