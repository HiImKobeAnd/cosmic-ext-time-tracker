use std::{
    ops::Index,
    rc::Rc,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use cosmic::{
    app,
    cosmic_config::CosmicConfigEntry,
    iced::{
        widget::{column, row},
        window, Length,
    },
    task,
    widget::{button, dropdown, icon, text_input},
    Element, Task,
};
use tracker_integrations::{ApiId, TimeEntry, TogglClient, Workspace};

use crate::{
    applet::{self, AppletModel},
    config::GlobalState,
};

pub struct TimerPage {
    pub state: GlobalState,
    state_handler: cosmic::cosmic_config::Config,
    current_workspace: Option<usize>,
}

#[derive(Debug, Clone)]
pub enum Message {
    // TaskTextChanged(String),
    GetWorkspaces,
    WorkspacesGotten(Option<Vec<Workspace>>),
    WorkspaceSelected(usize),
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
            row![reset_button]
        ])
        // .explain(cosmic::iced::Color::WHITE)
    }

    pub fn update(&mut self, message: Message) -> app::Task<Message> {
        match message {
            // Message::TaskTextChanged(task) => {
            // self.current_task = task;
            // Task::none()
            // }
            Message::GetWorkspaces => cosmic::task::future(async move {
                let workspaces = TogglClient::get_user_workspaces().await;
                match workspaces {
                    Ok(workspaces) => Message::WorkspacesGotten(Some(workspaces)),
                    Err(_) => Message::WorkspacesGotten(None),
                }
            }),
            Message::WorkspacesGotten(workspaces) => {
                if let Some(workspaces) = workspaces {
                    self.state
                        .set_workspaces(&self.state_handler, workspaces.clone());
                }
                Task::none()
            }
            Message::WorkspaceSelected(index) => {
                self.current_workspace = Some(index);
                self.state.set_selected_workspace(
                    &self.state_handler,
                    Some(self.state.workspaces[index].clone()),
                );
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
        (
            TimerPage {
                state,
                state_handler,
                // current_task: "".to_string(),
                current_workspace,
            },
            Task::done(Message::GetWorkspaces),
        )
    }
}
