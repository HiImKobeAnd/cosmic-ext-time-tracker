use crate::{
    applet::{self},
    config::GlobalState,
};
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
        Activity, Integration, Project, ProjectContext, TimeEntry, TimeEntryContext, Workspace,
    },
};

pub struct TimerPage {
    pub state: GlobalState,
    state_handler: cosmic::cosmic_config::Config,
    current_workspace: Option<usize>,
    current_project: Option<usize>,
    current_activity: Option<usize>,
    current_description: Option<String>,
    pub integration_client: Option<Arc<dyn TrackerIntegration>>,
}

#[derive(Debug, Clone)]
pub enum Message {
    GetWorkspaces,
    WorkspacesGotten(Option<Vec<Workspace>>),
    GetProjects(ProjectContext),
    ProjectsGotten(Option<Vec<Project>>),
    GetActivities,
    ActivitiesGotten(Option<Vec<Activity>>),
    WorkspaceSelected(usize),
    ProjectSelected(usize),
    ActivitySelected(usize),
    DescriptionChanged(String),
    GetExistingTimeEntry,
    ExistingTimeEntryGotten(Option<TimeEntry>),
}

impl From<Message> for applet::Message {
    fn from(message: Message) -> Self {
        applet::Message::TimerPage(message)
    }
}

impl TimerPage {
    pub fn view(&self) -> cosmic::Element<'_, Message> {
        let workspace_selector = dropdown::dropdown(
            self.state
                .workspaces
                .iter()
                .map(|x| x.name.clone())
                .collect::<Vec<String>>(),
            self.current_workspace,
            Message::WorkspaceSelected,
        );
        let project_selector = dropdown::dropdown(
            self.state
                .projects
                .iter()
                .map(|x| x.name.clone())
                .collect::<Vec<String>>(),
            self.current_project,
            Message::ProjectSelected,
        );
        let activity_selector = dropdown::dropdown(
            self.state
                .activities
                .iter()
                .map(|x| x.name.clone())
                .collect::<Vec<String>>(),
            self.current_activity,
            Message::ActivitySelected,
        );
        let description_input = text_input::text_input(
            "Description",
            self.current_description.clone().unwrap_or_default(),
        )
        .on_input(Message::DescriptionChanged);

        let refetch_existing_timer = button::icon(icon::from_name("object-rotate-left-symbolic"))
            .on_press(Message::GetExistingTimeEntry);

        let mut elements = Vec::new();
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
            Message::WorkspaceSelected(index) => {
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
            Message::ProjectSelected(index) => {
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
                    match time_entry.context {
                        TimeEntryContext::Kimai {
                            activity_id: _,
                            project_id,
                        } => {
                            if let Some(project_index) =
                                self.state.projects.iter().position(|p| p.id == project_id)
                            {
                                return cosmic::task::message(Message::ProjectSelected(
                                    project_index,
                                ));
                            };
                        }
                        TimeEntryContext::Toggl {
                            workspace_id,
                            project_id: _,
                        } => {
                            if let Some(workspace_index) = self
                                .state
                                .workspaces
                                .iter()
                                .position(|w| w.id == workspace_id)
                            {
                                return cosmic::task::message(Message::WorkspaceSelected(
                                    workspace_index,
                                ));
                            };
                        }
                    }
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
            Message::ActivitySelected(index) => {
                self.current_activity = Some(index);
                let _ = self.state.set_selected_activity(
                    &self.state_handler,
                    Some(self.state.activities[index].clone()),
                );
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

        TimerPage {
            state,
            state_handler,
            integration_client,
            current_workspace,
            current_project,
            current_activity,
            current_description,
        }
    }
}
