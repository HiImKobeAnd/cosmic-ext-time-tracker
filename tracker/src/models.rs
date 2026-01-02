use chrono::{DateTime, Duration, Local};

pub struct Project {
    id: String,
    name: String,
}
pub struct Tag {
    id: String,
    name: String,
}
pub struct TimeEntry {
    id: String,
    duration: Duration,
    start_time: DateTime<Local>, // !TODO Research what implications that using local will have
    stop_time: DateTime<Local>,  // !TODO Research what implications that using local will have
    project_id: String,
    tag_ids: Vec<String>,
}
