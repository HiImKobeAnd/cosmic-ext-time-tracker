use crate::{
    applet::{self},
    config::GlobalState,
};
use chrono::NaiveTime;
use cosmic::{
    app,
    iced::{widget::row, Alignment, Length},
    widget::{
        button, dropdown, icon,
        text::{self},
        text_input, Column,
    },
    Task,
};
use std::sync::Arc;
use tracker_api::{
    integration::TrackerIntegration,
    models::{Project, Scope, TimeEntry, TimeEntryUpdate},
};

pub struct TimerPage {
    pub state: GlobalState,
    pub integration_client: Option<Arc<dyn TrackerIntegration>>,
    state_handler: cosmic::cosmic_config::Config,
    current_scope: Option<usize>,
    current_project: Option<usize>,
    current_description: Option<String>,
    start_time_field_text: String,
    start_time_field_editable: bool,
}

#[derive(Debug, Clone)]
pub enum Message {
    GetScopes,
    ScopesGotten(Option<Vec<Scope>>),
    GetProjects,
    ProjectsGotten(Option<Vec<Project>>),
    ScopeChanged(usize),
    ProjectChanged(usize),
    DescriptionChanged(String),
    GetExistingTimeEntry,
    ExistingTimeEntryGotten(Option<TimeEntry>),
    StartTimeFieldTextChanged(String),
    StartTimeFieldSubmitted(String),
    ToggleStartTimeEditing(bool),
    StartTimeFieldUnfocused,
}

impl From<Message> for applet::Message {
    fn from(message: Message) -> Self {
        applet::Message::TimerPage(message)
    }
}

impl TimerPage {
    pub fn view(&self) -> cosmic::Element<'_, Message> {
        let start_time_field_placeholder = self
            .state
            .running_time_entry
            .as_ref()
            .map(|t| t.start_time.format("%H:%M").to_string())
            .unwrap_or("No running timer.".to_string());
        let start_time_field = text_input::editable_input(
            start_time_field_placeholder,
            &self.start_time_field_text,
            self.start_time_field_editable,
            Message::ToggleStartTimeEditing,
        )
        .on_input(Message::StartTimeFieldTextChanged)
        .on_submit(Message::StartTimeFieldSubmitted)
        .on_unfocus(Message::StartTimeFieldUnfocused);
        let scope_selector = dropdown::dropdown(
            self.state
                .scopes
                .iter()
                .map(|x| x.name.clone())
                .collect::<Vec<String>>(),
            self.current_scope,
            Message::ScopeChanged,
        );
        let project_selector = dropdown::dropdown(
            self.state
                .projects
                .iter()
                .filter(|p| {
                    self.state
                        .selected_scope
                        .as_ref()
                        .is_some_and(|selected_scope| p.scope_id == selected_scope.id)
                })
                .map(|x| x.name.clone())
                .collect::<Vec<String>>(),
            self.current_project,
            Message::ProjectChanged,
        );

        let description_input = text_input::text_input(
            "Description",
            self.current_description.clone().unwrap_or_default(),
        )
        .on_input(Message::DescriptionChanged);

        let refetch_existing_timer = button::icon(icon::from_name("object-rotate-left-symbolic"))
            .on_press(Message::GetExistingTimeEntry);

        Column::with_children(vec![
            row![
                text::body("Start").align_y(Alignment::Center),
                start_time_field
            ]
            .width(Length::Fill)
            .into(),
            text::body("Scope").width(Length::Fill).into(),
            scope_selector.width(Length::Fill).into(),
            text::body("Project").width(Length::Fill).into(),
            project_selector.width(Length::Fill).into(),
            text::body("Description").width(Length::Fill).into(),
            description_input.width(Length::Fill).into(),
            row![refetch_existing_timer].width(Length::Fill).into(),
        ])
        .into()
    }

    pub fn update(&mut self, message: Message) -> app::Task<Message> {
        match message {
            Message::GetScopes => {
                if let Some(client) = &self.integration_client {
                    let client = Arc::clone(client);
                    return cosmic::task::future(async move {
                        let scopes = client.get_all_scopes().await.ok();
                        Message::ScopesGotten(scopes)
                    });
                };
                Task::none()
            }
            Message::ScopesGotten(scopes) => {
                if let Some(scopes) = scopes {
                    let _ = self.state.set_scopes(&self.state_handler, scopes);
                }
                Task::none()
            }
            Message::GetProjects => {
                if let Some(client) = &self.integration_client {
                    let client = Arc::clone(client);
                    return cosmic::task::future(async move {
                        let projects = client.get_all_projects().await.ok();
                        Message::ProjectsGotten(projects)
                    });
                };
                Task::none()
            }
            Message::ProjectsGotten(projects) => {
                if let Some(projects) = projects {
                    let _ = self
                        .state
                        .set_projects(&self.state_handler, projects.clone());
                }
                Task::none()
            }
            Message::ScopeChanged(index) => {
                self.current_scope = Some(index);
                let _ = self.state.set_selected_scope(
                    &self.state_handler,
                    Some(self.state.scopes[index].clone()),
                );
                let _ = self.state.set_selected_project(&self.state_handler, None);
                Task::none()
            }
            Message::ProjectChanged(index) => {
                self.current_project = Some(index);
                let _ = self.state.set_selected_project(
                    &self.state_handler,
                    Some(self.state.projects[index].clone()),
                );
                Task::none()
            }
            Message::DescriptionChanged(description) => {
                let desc = match description.is_empty() {
                    true => None,
                    false => Some(description),
                };
                self.current_description = desc.clone();
                let _ = self
                    .state
                    .set_current_description(&self.state_handler, desc);
                Task::none()
            }
            Message::GetExistingTimeEntry => {
                if let Some(client) = &self.integration_client {
                    let client = Arc::clone(client);
                    return cosmic::task::future(async move {
                        let time_entry = client.get_current_time_entry().await;
                        match time_entry {
                            Ok(entry) => Message::ExistingTimeEntryGotten(entry),
                            Err(_) => Message::ExistingTimeEntryGotten(None),
                        }
                    });
                };
                Task::none()
            }
            Message::ExistingTimeEntryGotten(time_entry) => {
                let _ = self
                    .state
                    .set_running_time_entry(&self.state_handler, time_entry.clone());
                Task::none()
            }
            Message::StartTimeFieldTextChanged(text) => {
                self.start_time_field_text = text;
                Task::none()
            }
            Message::StartTimeFieldSubmitted(text) => {
                let Some(client) = &self.integration_client else {
                    return Task::none();
                };
                let Some(parsed_time) = NaiveTime::parse_from_str(&text, "%H:%M").ok() else {
                    return Task::none();
                };
                let Some(time_entry) = &self.state.running_time_entry else {
                    return Task::none();
                };

                let time_entry = time_entry.clone();
                let Some(new_start_time) = time_entry.start_time.with_time(parsed_time).single()
                else {
                    return Task::none();
                };

                let time_entry_update = TimeEntryUpdate {
                    billable: time_entry.billable,
                    description: time_entry.description.clone(),
                    start_time: new_start_time,
                    stop_time: time_entry.stop_time,
                };

                let client = Arc::clone(client);
                cosmic::task::future(async move {
                    let time_entry = client
                        .update_time_entry(&time_entry, &time_entry_update)
                        .await
                        .ok();
                    Message::ExistingTimeEntryGotten(time_entry)
                })
            }
            Message::ToggleStartTimeEditing(editiable) => {
                if editiable {
                    if let Some(running_time_entry) = &self.state.running_time_entry {
                        self.start_time_field_text =
                            running_time_entry.start_time.format("%H:%M").to_string();
                        self.start_time_field_editable = editiable;
                    } else {
                        self.start_time_field_editable = false;
                        self.start_time_field_text.clear();
                    }
                }
                Task::none()
            }
            Message::StartTimeFieldUnfocused => {
                self.start_time_field_editable = false;
                self.start_time_field_text.clear();
                Task::none()
            }
        }
    }
    pub fn new(
        state: GlobalState,
        state_handler: cosmic::cosmic_config::Config,
        integration_client: Option<Arc<dyn TrackerIntegration>>,
    ) -> Self {
        let mut current_scope = None;
        if let Some(selected_scope) = &state.selected_scope {
            current_scope = state.scopes.iter().position(|w| w.id == selected_scope.id);
        }
        let mut current_project = None;
        if let Some(selected_project) = &state.selected_project {
            current_project = state
                .projects
                .iter()
                .position(|p| p.id == selected_project.id);
        }
        let current_description = state.current_description.clone();

        TimerPage {
            state,
            state_handler,
            integration_client,
            current_scope,
            current_project,
            current_description,
            start_time_field_text: String::default(),
            start_time_field_editable: false,
        }
    }
}
