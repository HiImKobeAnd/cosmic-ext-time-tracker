use std::sync::Arc;

use cosmic::{
    app,
    iced::Length,
    iced_widget::{pick_list, text_input},
    theme::CosmicTheme,
    widget::{button, Column},
    Element, Task,
};
use tracker_integrations::{
    authentication::{
        get_integration_url, remove_api_key, remove_integration_url, set_api_key,
        set_integration_url,
    },
    integration::TrackerIntegration,
    models::Integration,
};

use crate::{applet, config::GlobalState};

pub struct SettingsPage {
    pub state: GlobalState,
    pub state_handler: cosmic::cosmic_config::Config,
    trackers: Vec<Integration>,
    selected_tracker: Option<Integration>,
    api_key: String,
    integration_url: String,
    pub integration_client: Option<Arc<dyn TrackerIntegration>>,
}

#[derive(Debug, Clone)]
pub enum Message {
    TrackerSelected(Integration),
    APIKeyInput(String),
    APIKeySubmitted,
    IntegrationUrlInput(String),
    IntegrationUrlSubmitted,
    // ValidateAuthentication,
    SaveCredentials,
    CredentialsSaved,
    RemoveCredentials,
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
        let integration_url_input = text_input("https://kimai.example.net", &self.integration_url)
            .on_input(Message::IntegrationUrlInput)
            .on_submit(Message::IntegrationUrlSubmitted);

        let api_key_input = text_input("Input API Key", &"*".repeat(self.api_key.len()))
            .on_input(Message::APIKeyInput)
            .on_submit(Message::APIKeySubmitted);

        let mut elements = Vec::new();

        elements.push(tracker_selector.width(Length::Fill).into());

        if let Some(selected_tracker) = &self.state.selected_tracker {
            match selected_tracker {
                tracker_integrations::models::Integration::TogglIntegration => {
                    elements.push(api_key_input.width(Length::Fill).into());
                }
                tracker_integrations::models::Integration::KimaiIntegration => {
                    elements.push(integration_url_input.width(Length::Fill).into());
                    elements.push(api_key_input.width(Length::Fill).into());
                }
            }
        }

        let button = if self.integration_client.is_some() {
            button::destructive("Unauthenticate").on_press(Message::RemoveCredentials)
        } else {
            button::suggested("Authenctiacte").on_press(Message::SaveCredentials)
        };
        elements.push(button.into());

        Element::from(Column::new().extend(elements))
    }

    pub fn update(&mut self, message: Message) -> app::Task<Message> {
        match message {
            Message::TrackerSelected(selected) => {
                self.selected_tracker = Some(selected.clone());
                let _ = self
                    .state
                    .set_selected_tracker(&self.state_handler, Some(selected));
                Task::none()
            }
            Message::APIKeyInput(api_key) => {
                self.api_key = api_key;
                Task::none()
            }
            Message::APIKeySubmitted => {
                if let Some(selected_tracker) = &self.selected_tracker {
                    let _ = set_api_key(selected_tracker, self.api_key.clone());
                }
                Task::none()
            }
            Message::IntegrationUrlInput(integration_url) => {
                self.integration_url = integration_url;
                Task::none()
            }
            Message::IntegrationUrlSubmitted => {
                if let Some(selected_tracker) = &self.selected_tracker {
                    let _ = set_integration_url(selected_tracker, self.integration_url.clone());
                }
                Task::none()
            }
            Message::SaveCredentials => {
                let Some(selected_tracker) = &self.state.selected_tracker else {
                    return Task::none();
                };

                let mut tasks: Vec<Task<Message>> = Vec::new();
                match selected_tracker {
                    Integration::TogglIntegration => {
                        tasks.push(cosmic::task::message(Message::APIKeySubmitted));
                    }
                    Integration::KimaiIntegration => {
                        tasks.push(cosmic::task::message(Message::APIKeySubmitted));
                        tasks.push(cosmic::task::message(Message::IntegrationUrlSubmitted));
                    }
                }
                cosmic::task::batch(tasks).chain(cosmic::task::message(Message::CredentialsSaved))
            }
            Message::CredentialsSaved => Task::none(),
            Message::RemoveCredentials => {
                let Some(selected_tracker) = &self.state.selected_tracker else {
                    return Task::none();
                };

                match selected_tracker {
                    Integration::TogglIntegration => {
                        let _ = remove_api_key(selected_tracker);
                    }
                    Integration::KimaiIntegration => {
                        let _ = remove_api_key(selected_tracker);
                        let _ = remove_integration_url(selected_tracker);
                    }
                };
                cosmic::task::message(Message::CredentialsSaved)
            }
        }
    }

    pub fn new(
        state: GlobalState,
        state_handler: cosmic::cosmic_config::Config,
        integration_client: Option<Arc<dyn TrackerIntegration>>,
    ) -> Self {
        let selected_tracker = state.selected_tracker.clone();
        let integration_url = state
            .selected_tracker
            .as_ref()
            .and_then(|tracker| get_integration_url(tracker).ok())
            .unwrap_or_default();
        SettingsPage {
            state,
            state_handler,
            trackers: vec![Integration::TogglIntegration, Integration::KimaiIntegration],
            selected_tracker,
            api_key: String::default(),
            integration_url: integration_url,
            integration_client,
        }
    }
}
