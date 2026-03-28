use crate::{
    applet::{self},
    config::GlobalState,
};
use chrono::NaiveTime;
use cosmic::{
    app,
    iced::{widget::row, Length},
    widget::{button, dropdown, icon, text_input, Column},
    Element, Task,
};
use std::sync::Arc;
use tracker_integrations::{
    integration::TrackerIntegration,
    models::{
        Activity, Integration, Project, ProjectContext, TimeEntry, TimeEntryContext,
        TimeEntryUpdate, Workspace,
    },
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
}

#[derive(Debug, Clone)]
pub enum Message {
    GetWorkspaces,
    WorkspacesGotten(Option<Vec<Workspace>>),
    GetProjects(ProjectContext),
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
    StartTimeFieldUnfocused,
}

impl From<Message> for applet::Message {
    fn from(message: Message) -> Self {
        applet::Message::TimerPage(message)
    }
}

impl TimerPage {
    pub fn view(&self) -> cosmic::Element<'_, Message> {
        let start_time_field = text_input("No running timer.", self.start_time_field_text.clone())
            .label("From:")
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
                .map(|x| x.name.clone())
                .collect::<Vec<String>>(),
            self.current_project,
            Message::ProjectChanged,
        );
        let activity_selector = dropdown::dropdown(
            self.state
                .activities
                .iter()
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
        elements.push(row![start_time_field].width(Length::Fill).into());
        if let Some(selected_tracker) = &self.state.selected_tracker {
            match selected_tracker {
                tracker_integrations::models::Integration::TogglIntegration => {
                    elements.push(workspace_selector.width(Length::Fill).into());
                    elements.push(project_selector.width(Length::Fill).into());
                    elements.push(description_input.width(Length::Fill).into());
                }
                tracker_integrations::models::Integration::KimaiIntegration => {
                    elements.push(project_selector.width(Length::Fill).into());
                    elements.push(activity_selector.width(Length::Fill).into());
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
                        let workspaces = client.get_user_workspaces().await.ok();
                        Message::WorkspacesGotten(workspaces)
                    });
                };
                Task::none()
            }
            Message::WorkspacesGotten(workspaces) => {
                if let Some(workspaces) = workspaces {
                    let _ = self
                        .state
                        .set_workspaces(&self.state_handler, workspaces.clone());
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
                cosmic::task::message(Message::GetProjects(ProjectContext::Toggl {
                    workspace_id: self.state.workspaces[index].id.clone(),
                }))
            }
            Message::ProjectChanged(index) => {
                self.current_project = Some(index);
                let _ = self.state.set_selected_project(
                    &self.state_handler,
                    Some(self.state.projects[index].clone()),
                );
                if let Some(selected_tracker) = &self.state.selected_tracker {
                    match selected_tracker {
                        Integration::KimaiIntegration => {
                            return cosmic::task::message(Message::GetActivities);
                        }
                        _ => return Task::none(),
                    }
                }
                Task::none()
            }
            Message::GetProjects(context) => {
                if let Some(client) = &self.integration_client {
                    let client = Arc::clone(client);
                    return cosmic::task::future(async move {
                        let projects = client.get_projects(context).await.ok();
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

                if let Some(time_entry) = time_entry {
                    self.start_time_field_text = time_entry.start_time.format("%H:%M").to_string();

                    // Triggers a fetch for either workspaces or projects based on the type of
                    // timeentrycontext
                    // match time_entry.context {
                    //     TimeEntryContext::Kimai {
                    //         activity_id: _,
                    //         project_id,
                    //     } => {
                    //         if let Some(project_index) =
                    //             self.state.projects.iter().position(|p| p.id == project_id)
                    //         {
                    //             return cosmic::task::message(Message::ProjectChanged(
                    //                 project_index,
                    //             ));
                    //         };
                    //     }
                    //     TimeEntryContext::Toggl {
                    //         workspace_id,
                    //         project_id: _,
                    //     } => {
                    //         if let Some(workspace_index) = self
                    //             .state
                    //             .workspaces
                    //             .iter()
                    //             .position(|w| w.id == workspace_id)
                    //         {
                    //             return cosmic::task::message(Message::WorkspaceChanged(
                    //                 workspace_index,
                    //             ));
                    //         };
                    //     }
                    // }
                }
                Task::none()
            }
            Message::GetActivities => {
                if let Some(client) = &self.integration_client {
                    let client = Arc::clone(client);
                    if let Some(selected_project) = &self.state.selected_project {
                        let project_id = selected_project.id.clone();
                        return cosmic::task::future(async move {
                            let activities = client.get_project_activities(project_id).await.ok();
                            Message::ActivitiesGotten(activities)
                        });
                    }
                }
                Task::none()
            }
            Message::ActivitiesGotten(activities) => {
                if let Some(activities) = activities {
                    let _ = self
                        .state
                        .set_activities(&self.state_handler, activities.clone());
                }
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
            Message::StartTimeFieldUnfocused => {
                self.start_time_field_text = self
                    .state
                    .running_time_entry
                    .clone()
                    .map(|e| e.start_time.format("%H:%M").to_string())
                    .unwrap_or("No running timer".to_string());
                Task::none()
            }
        }
    }
    pub fn new(
        integration_client: Option<Arc<dyn TrackerIntegration>>,
        state: GlobalState,
        state_handler: cosmic::cosmic_config::Config,
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

        let start_time_field_text = state
            .running_time_entry
            .clone()
            .map(|e| e.start_time.format("%H:%M").to_string())
            .unwrap_or("No running timer".to_string());

        TimerPage {
            state,
            state_handler,
            integration_client,
            current_workspace,
            current_project,
            current_activity,
            current_description,
            start_time_field_text,
        }
    }
}
