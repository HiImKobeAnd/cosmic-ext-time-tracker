use std::{
    rc::Rc,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use cosmic::{
    app,
    iced::{
        widget::{column, row},
        window, Length,
    },
    task,
    widget::{button, dropdown, icon, text_input},
    Element, Task,
};
use tracker_integrations::{ApiId, TimeEntry, TogglClient, Workspace};

use crate::applet::{self, AppletModel};

pub struct TimerPage {
    current_task: String,
    timer_running: Arc<AtomicBool>,
    workspaces: Vec<Workspace>,
    current_workspace: Option<usize>,
}

#[derive(Debug, Clone)]
pub enum Message {
    TaskTextChanged(String),
    GetWorkspaces,
    WorkspacesGotten(Option<Vec<Workspace>>),
    WorkspaceSelected(usize),
    NotifyWorkspaceChanged(ApiId),
}

impl From<Message> for applet::Message {
    fn from(message: Message) -> Self {
        applet::Message::TimerPage(message)
    }
}

impl TimerPage {
    pub fn view(&self) -> cosmic::Element<'_, Message> {
        let task_selector = text_input::text_input("Task", self.current_task.clone())
            .on_input(Message::TaskTextChanged);

        let workspace_selector = dropdown::dropdown(
            self.workspaces
                .iter()
                .map(|x| x.name.clone())
                .collect::<Vec<String>>(),
            self.current_workspace,
            Message::WorkspaceSelected,
        );

        let toggle_timer_button = button::icon(match self.timer_running.load(Ordering::Relaxed) {
            true => icon::from_name("media-playback-stop-symbolic"),
            false => icon::from_name("media-playback-start-symbolic"),
        })
        // .on_press(Message::ToggleTimer)
        .class(cosmic::theme::Button::AppletIcon);

        let reset_button = button::icon(icon::from_name("object-rotate-left-symbolic"))
            // .on_press(Message::ResetTimer)
            .class(cosmic::theme::Button::AppletIcon);

        Element::from(column![
            task_selector.width(Length::Fill),
            workspace_selector.width(Length::Fill),
            row![toggle_timer_button, reset_button]
        ])
        // .explain(cosmic::iced::Color::WHITE)
    }

    pub fn update(&mut self, message: Message) -> app::Task<Message> {
        match message {
            Message::TaskTextChanged(task) => {
                self.current_task = task;
                Task::none()
            }
            Message::GetWorkspaces => cosmic::task::future(async move {
                let workspaces = TogglClient::get_user_workspaces().await;
                match workspaces {
                    Ok(workspaces) => Message::WorkspacesGotten(Some(workspaces)),
                    Err(_) => Message::WorkspacesGotten(None),
                }
            }),
            Message::WorkspacesGotten(workspaces) => {
                if let Some(workspaces) = workspaces {
                    self.workspaces = workspaces
                }
                Task::none()
            }
            Message::WorkspaceSelected(index) => {
                self.current_workspace = Some(index);
                let selected_workspace = self.workspaces[index].clone();
                cosmic::task::message(Message::NotifyWorkspaceChanged(
                    selected_workspace.id.clone(),
                ))
            }
            Message::NotifyWorkspaceChanged(api_id) => Task::none(),
        }
    }
    pub fn new(timer_running: Arc<AtomicBool>) -> (Self, Task<Message>) {
        (
            TimerPage {
                current_task: "".to_string(),
                timer_running,
                current_workspace: None,
                workspaces: Vec::new(),
            },
            Task::done(Message::GetWorkspaces),
        )
    }
}
