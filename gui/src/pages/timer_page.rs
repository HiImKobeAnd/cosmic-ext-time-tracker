use crate::{
    applet::{self},
    config::GlobalState,
};
use chrono::NaiveTime;
use cosmic::{
    app,
    iced::{widget::row, Alignment, Length},
    iced_winit::graphics::text::cosmic_text::Align,
    widget::{
        button, dropdown, icon,
        text::{self, caption},
        text_input, Column,
    },
    Element, Task,
};
use std::sync::Arc;
use tracker_integrations::{
    integration::TrackerIntegration,
    models::{Activity, Project, ProjectContext, TimeEntry, TimeEntryUpdate, Workspace},
};

pub struct TimerPage {
    pub state: GlobalState,
    pub integration_client: Option<Arc<dyn TrackerIntegration>>,
    state_handler: cosmic::cosmic_config::Config,
    current_workspace: Option<usize>,
    current_project: Option<usize>,
    current_activity: Option<usize>,
    current_description: Option<String>,
    start_time_field_text: String,
    start_time_field_editable: bool,
}

#[derive(Debug, Clone)]
pub enum Message {
    GetWorkspaces,
    WorkspacesGotten(Option<Vec<Workspace>>),
    GetProjects,
    ProjectsGotten(Option<Vec<Project>>),
    GetActivities,
    ActivitiesGotten(Option<Vec<Activity>>),
    WorkspaceChanged(usize),
    ProjectChanged(usize),
    ActivityChanged(usize),
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
        let workspace_selector = dropdown::dropdown(
            self.state
                .workspaces
                .iter()
                .map(|x| x.name.clone())
                .collect::<Vec<String>>(),
            self.current_workspace,
            Message::WorkspaceChanged,
        );
        let project_selector = dropdown::dropdown(
            self.state
                .projects
                .iter()
                .filter(|x| {
                    if let ProjectContext::Toggl { workspace_id } = &x.context {
                        return self.state.selected_workspace.as_ref().is_some_and(
                            |selected_workspace| *workspace_id == selected_workspace.id,
                        );
                    }
                    true
                })
                .map(|x| x.name.clone())
                .collect::<Vec<String>>(),
            self.current_project,
            Message::ProjectChanged,
        );
        let activity_selector = dropdown::dropdown(
            self.state
                .activities
                .iter()
                .filter(|x| {
                    if let Some(selected_project) = &self.state.selected_project {
                        x.project_id == selected_project.id
                    } else {
                        false
                    }
                })
                .map(|x| x.name.clone())
                .collect::<Vec<String>>(),
            self.current_activity,
            Message::ActivityChanged,
        );
        let description_input = text_input::text_input(
            "Description",
            self.current_description.clone().unwrap_or_default(),
        )
        .on_input(Message::DescriptionChanged);

        let refetch_existing_timer = button::icon(icon::from_name("object-rotate-left-symbolic"))
            .on_press(Message::GetExistingTimeEntry);

        let mut elements = Vec::new();
        elements.push(
            row![
                text::body("Start").align_y(Alignment::Center),
                start_time_field
            ]
            .width(Length::Fill)
            .into(),
        );
        if let Some(selected_tracker) = &self.state.selected_tracker {
            match selected_tracker {
                tracker_integrations::models::Integration::TogglIntegration => {
                    // elements.push(text::body("Workspace").width(Length::Fill).into());
                    elements.push(
                        row![
                            text::body("workspace"),
                            workspace_selector.width(Length::Fill)
                        ]
                        .into(),
                    );
                    elements.push(text::body("Project").width(Length::Fill).into());
                    elements.push(project_selector.width(Length::Fill).into());
                    elements.push(text::body("Description").width(Length::Fill).into());
                    elements.push(description_input.width(Length::Fill).into());
                }
                tracker_integrations::models::Integration::KimaiIntegration => {
                    elements.push(
                        row![
                            text::body("Project").align_y(Alignment::Center),
                            project_selector.width(Length::Fill)
                        ]
                        .into(),
                    );
                    elements.push(
                        row![
                            text::body("Activity").align_y(Alignment::Center),
                            activity_selector.width(Length::Fill)
                        ]
                        .into(),
                    );
                    elements.push(text::body("Description").width(Length::Fill).into());
                    elements.push(description_input.width(Length::Fill).into());
                }
            }
        }
        elements.push(row![refetch_existing_timer].width(Length::Fill).into());

        Element::from(Column::new().extend(elements))
        // .explain(cosmic::iced::Color::WHITE)
    }

    pub fn update(&mut self, message: Message) -> app::Task<Message> {
        match message {
            Message::GetWorkspaces => {
                if let Some(client) = &self.integration_client {
                    let client = Arc::clone(client);
                    return cosmic::task::future(async move {
                        let workspaces = client.get_all_workspaces().await.ok();
                        Message::WorkspacesGotten(workspaces)
                    });
                };
                Task::none()
            }
            Message::WorkspacesGotten(workspaces) => {
                if let Some(workspaces) = workspaces {
                    let _ = self.state.set_workspaces(&self.state_handler, workspaces);
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
            Message::GetActivities => {
                if let Some(client) = &self.integration_client {
                    let client = Arc::clone(client);
                    return cosmic::task::future(async move {
                        let activities = client.get_all_activities().await.ok();
                        Message::ActivitiesGotten(activities)
                    });
                }
                Task::none()
            }
            Message::ActivitiesGotten(activities) => {
                if let Some(activities) = activities {
                    let _ = self.state.set_activities(&self.state_handler, activities);
                }
                Task::none()
            }
            Message::WorkspaceChanged(index) => {
                self.current_workspace = Some(index);
                let _ = self.state.set_selected_workspace(
                    &self.state_handler,
                    Some(self.state.workspaces[index].clone()),
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
                let _ = self.state.set_selected_activity(&self.state_handler, None);
                Task::none()
            }
            Message::ActivityChanged(index) => {
                self.current_activity = Some(index);
                let _ = self.state.set_selected_activity(
                    &self.state_handler,
                    Some(self.state.activities[index].clone()),
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
        let mut current_workspace = None;
        if let Some(selected_workspace) = &state.selected_workspace {
            current_workspace = state
                .workspaces
                .iter()
                .position(|w| w.id == selected_workspace.id);
        }
        let mut current_project = None;
        if let Some(selected_project) = &state.selected_project {
            current_project = state
                .projects
                .iter()
                .position(|p| p.id == selected_project.id);
        }
        let mut current_activity = None;
        if let Some(selected_activity) = &state.selected_activity {
            current_activity = state
                .activities
                .iter()
                .position(|p| p.id == selected_activity.id);
        }
        let current_description = state.current_description.clone();

        TimerPage {
            state,
            state_handler,
            integration_client,
            current_workspace,
            current_project,
            current_activity,
            current_description,
            start_time_field_text: String::default(),
            start_time_field_editable: false,
        }
    }
}
