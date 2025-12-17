use std::{fs::File, process, time::Duration};

use cosmic::{
    action, app,
    iced::{
        widget::{column, row},
        window, Length,
    },
    task,
    widget::{button, dropdown, icon, text, text_input},
    Element, Task,
};

pub struct TimeEntriesPage {
    time_entries: Vec<String>,
        counter: i32,
    counter_text: String,
}

#[derive(Debug, Clone)]
pub enum Message {
    Test,
    GetAllEntries,
}

impl TimeEntriesPage {
    pub fn view(&self) -> cosmic::Element<'_, Message> {
        let reset_button = button::icon(icon::from_name("object-rotate-left-symbolic"))
            .on_press(Message::GetAllEntries)
            .class(cosmic::theme::Button::AppletIcon);
        let counter_button =
            button::custom(text::text(&self.counter_text)).on_press(Message::GetAllEntries);
        column![counter_button, reset_button].into()
    }

    pub fn update(&mut self, message: Message) -> app::Task<Message> {
        match message {
            Message::Test => {
                self.counter += 1;
                self.counter_text = format!("Clicked {} times", self.counter);
                app::Task::none()
            }
            Message::GetAllEntries => cosmic::task::future(async move {
                tokio::time::sleep(Duration::from_secs(1)).await;
                Message::Test
            }),
        }
    }

    pub fn new() -> Self {
        TimeEntriesPage {
            time_entries: vec![],
            counter: 0,
            counter_text: "Clicked 0 times".to_string(),
        }
    }
}
