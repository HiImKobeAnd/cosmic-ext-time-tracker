use cosmic::{
    app,
    iced::{widget::column, Length},
    widget::{dropdown, text_input},
    Element, Task,
};
use tracker_integrations::{Project, TogglClient, Workspace};

use crate::{
    applet::{self},
    config::GlobalState,
};

pub struct TimerPage {
    pub state: GlobalState,
    state_handler: cosmic::cosmic_config::Config,
    current_workspace: Option<usize>,
    current_project: Option<usize>,
    current_description: Option<String>,
    toggl_client: TogglClient,
}

#[derive(Debug, Clone)]
pub enum Message {
    // TaskTextChanged(String),
    GetWorkspaces,
    WorkspacesGotten(Option<Vec<Workspace>>),
    WorkspaceSelected(usize),
    ProjectSelected(usize),
    GetProjectsForWorkspace(Workspace),
    ProjectsGotten(Option<Vec<Project>>),
    DescriptionChanged(String),
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
                .projects_for_selected_workspace
                .iter()
                .map(|x| x.name.clone())
                .collect::<Vec<String>>(),
            self.current_project,
            Message::ProjectSelected,
        );

        let description_input = text_input::text_input(
            "Description",
            self.current_description.clone().unwrap_or_default(),
        )
        .on_input(Message::DescriptionChanged);

        Element::from(column![
            workspace_selector.width(Length::Fill),
            project_selector.width(Length::Fill),
            description_input.width(Length::Fill),
        ])
        // .explain(cosmic::iced::Color::WHITE)
    }

    pub fn update(&mut self, message: Message) -> app::Task<Message> {
        match message {
            Message::GetWorkspaces => {
                let client = self.toggl_client.clone();
                cosmic::task::future(async move {
                    let workspaces = client.get_user_workspaces().await;
                    match workspaces {
                        Ok(workspaces) => Message::WorkspacesGotten(Some(workspaces)),
                        Err(_) => Message::WorkspacesGotten(None),
                    }
                })
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
                Task::done(cosmic::Action::App(Message::GetProjectsForWorkspace(
                    self.state.workspaces[index].clone(),
                )))
            }
            Message::ProjectSelected(index) => {
                self.current_project = Some(index);
                let _ = self.state.set_selected_project(
                    &self.state_handler,
                    Some(self.state.projects_for_selected_workspace[index].clone()),
                );
                Task::none()
            }
            Message::GetProjectsForWorkspace(workspace) => {
                let client = self.toggl_client.clone();
                cosmic::task::future(async move {
                    let projects = client.get_workspace_projects(workspace.id).await;
                    match projects {
                        Ok(projects) => Message::ProjectsGotten(Some(projects)),
                        Err(_) => Message::ProjectsGotten(None),
                    }
                })
            }
            Message::ProjectsGotten(projects) => {
                if let Some(projects) = projects {
                    let _ = self
                        .state
                        .set_projects_for_selected_workspace(&self.state_handler, projects.clone());
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
        }
    }
    pub fn new(
        toggl_client: TogglClient,
        state: GlobalState,
        state_handler: cosmic::cosmic_config::Config,
    ) -> (Self, Task<Message>) {
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
                .projects_for_selected_workspace
                .iter()
                .position(|p| p.id == selected_project.id);
        }
        let current_description = state.current_description.clone();

        (
            TimerPage {
                toggl_client,
                state,
                state_handler,
                current_workspace,
                current_project,
                current_description: current_description,
            },
            Task::done(Message::GetWorkspaces),
        )
    }
}
