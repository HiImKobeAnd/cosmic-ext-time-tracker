use cosmic::{
    app,
    iced::{
        widget::{column, row},
        Length,
    },
    widget::{button, dropdown, icon},
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
}

impl From<Message> for applet::Message {
    fn from(message: Message) -> Self {
        applet::Message::TimerPage(message)
    }
}

impl TimerPage {
    pub fn view(&self) -> cosmic::Element<'_, Message> {
        // let task_selector = text_input::text_input("Task", self.current_task.clone())
        // .on_input(Message::TaskTextChanged);

        let workspace_selector = dropdown::dropdown(
            self.state
                .workspaces
                .iter()
                .map(|x| x.name.clone())
                .collect::<Vec<String>>(),
            self.current_workspace,
            Message::WorkspaceSelected,
        );
        let projects_selector = dropdown::dropdown(
            self.state
                .projects_for_selected_workspace
                .iter()
                .map(|x| x.name.clone())
                .collect::<Vec<String>>(),
            self.current_project,
            Message::ProjectSelected,
        );
        // let toggle_timer_button = button::icon(match self.timer_running {
        //     true => icon::from_name("media-playback-stop-symbolic"),
        //     false => icon::from_name("media-playback-start-symbolic"),
        // })
        // .on_press(Message::ToggleTimer)
        // .class(cosmic::theme::Button::AppletIcon);

        let reset_button = button::icon(icon::from_name("object-rotate-left-symbolic"))
            // .on_press(Message::ResetTimer)
            .class(cosmic::theme::Button::AppletIcon);

        Element::from(column![
            // task_selector.width(Length::Fill),
            workspace_selector.width(Length::Fill),
            projects_selector.width(Length::Fill),
            row![reset_button]
        ])
        // .explain(cosmic::iced::Color::WHITE)
    }

    pub fn update(&mut self, message: Message) -> app::Task<Message> {
        match message {
            Message::GetWorkspaces => cosmic::task::future(async move {
                let workspaces = TogglClient::get_user_workspaces().await;
                match workspaces {
                    Ok(workspaces) => Message::WorkspacesGotten(Some(workspaces)),
                    Err(_) => Message::WorkspacesGotten(None),
                }
            }),
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
            Message::GetProjectsForWorkspace(workspace) => cosmic::task::future(async move {
                let projects = TogglClient::get_workspace_projects(workspace.id).await;
                match projects {
                    Ok(projects) => Message::ProjectsGotten(Some(projects)),
                    Err(_) => Message::ProjectsGotten(None),
                }
            }),
            Message::ProjectsGotten(projects) => {
                if let Some(projects) = projects {
                    let _ = self
                        .state
                        .set_projects_for_selected_workspace(&self.state_handler, projects.clone());
                }
                Task::none()
            }
        }
    }
    pub fn new(
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
        (
            TimerPage {
                state,
                state_handler,
                // current_task: "".to_string(),
                current_workspace,
                current_project,
            },
            Task::done(Message::GetWorkspaces),
        )
    }
}
