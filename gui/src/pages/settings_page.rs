use cosmic::{
    app,
    iced::{widget::column, Length},
    iced_widget::{pick_list, text_input},
    Element, Task,
};
use tracker_integrations::{set_api_key, Integration};

use crate::{applet, config::GlobalState};

pub struct SettingsPage {
    pub state: GlobalState,
    pub state_handler: cosmic::cosmic_config::Config,
    trackers: Vec<Integration>,
    selected_tracker: Option<Integration>,
    api_key: String,
}

#[derive(Debug, Clone)]
pub enum Message {
    TrackerSelected(Integration),
    APIKeyInput(String),
    APIKeySubmitted,
}

impl From<Message> for applet::Message {
    fn from(message: Message) -> Self {
        applet::Message::SettingsPage(message)
    }
}

impl SettingsPage {
    pub fn view(&self) -> cosmic::Element<'_, Message> {
        let tracker_selector = pick_list(
            self.trackers.clone(),
            self.selected_tracker.as_ref(),
            Message::TrackerSelected,
        )
        .placeholder("Select Tracker.");

        let api_key_input = text_input("Input API Key", &"*".repeat(self.api_key.len()))
            .on_input(Message::APIKeyInput)
            .on_submit(Message::APIKeySubmitted);

        Element::from(column![tracker_selector, api_key_input].height(Length::Fixed(500.)))
    }

    pub fn update(&mut self, message: Message) -> app::Task<Message> {
        match message {
            Message::TrackerSelected(selected) => {
                self.selected_tracker = Some(selected.clone());
                self.state
                    .set_selected_tracker(&self.state_handler, Some(selected));
                Task::none()
            }
            Message::APIKeyInput(api_key) => {
                self.api_key = api_key;
                Task::none()
            }
            Message::APIKeySubmitted => {
                if let Some(selected_tracker) = &self.selected_tracker {
                    set_api_key(selected_tracker, self.api_key.clone());
                }
                Task::none()
            }
        }
    }

    pub fn new(state: GlobalState, state_handler: cosmic::cosmic_config::Config) -> Self {
        let selected_tracker = state.selected_tracker.clone();
        SettingsPage {
            state,
            state_handler,
            trackers: vec![Integration::TogglIntegration],
            selected_tracker,
            api_key: String::default(),
        }
    }
}
