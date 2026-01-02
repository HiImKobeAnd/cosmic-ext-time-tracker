use cosmic::{
    app,
    iced::{
        widget::{column, row},
        window, Length,
    },
    widget::{button, dropdown, icon, text_input},
    Element, Task,
};

pub struct TimerPage {
    current_task: String,
    current_tag: Option<usize>,
    project_selections: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum Message {
    TaskTextChanged(String),
    TagChanged(usize),
}

impl TimerPage {
    pub fn view(&self) -> cosmic::Element<'_, Message> {
        let task_selector = text_input::text_input("Task", self.current_task.clone())
            .on_input(Message::TaskTextChanged);

        let project_selector = dropdown::dropdown(
            self.project_selections.clone(),
            self.current_tag,
            Message::TagChanged,
        );

        // let timer = text::text(format_duration(&self.timer_duration));

        // let toggle_timer_button = button::icon(if self.timer_running {
        // icon::from_name("media-playback-stop-symbolic")
        // } else {
        // icon::from_name("media-playback-start-symbolic")
        // })
        // .on_press(Message::ToggleTimer)
        // .class(cosmic::theme::Button::AppletIcon);

        let reset_button = button::icon(icon::from_name("object-rotate-left-symbolic"))
            // .on_press(Message::ResetTimer)
            .class(cosmic::theme::Button::AppletIcon);

        Element::from(column![
            task_selector.width(Length::Fill),
            project_selector.width(Length::Fill),
            row![reset_button]
        ])
        // .explain(cosmic::iced::Color::WHITE)
    }

    pub fn update(&mut self, message: Message) -> app::Task<Message> {
        match message {
            Message::TaskTextChanged(task) => {
                self.current_task = task;
                Task::none()
            }
            Message::TagChanged(tag) => {
                self.current_tag = Some(tag);
                Task::none()
            }
        }
    }
    pub fn new() -> Self {
        TimerPage {
            current_task: "".to_string(),
            current_tag: None,
            project_selections: vec![
                "Systemudvikling".to_string(),
                "Programmering".to_string(),
                "Teknologi".to_string(),
            ],
        }
    }
}
