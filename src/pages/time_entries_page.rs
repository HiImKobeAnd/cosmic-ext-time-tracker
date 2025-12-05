use cosmic::{
    app,
    iced::{
        widget::{column, row},
        window, Length,
    },
    widget::{button, dropdown, icon, text_input},
    Element, Task,
};

use crate::models::TimeEntry;

pub struct TimeEntriesPage {
    time_entries: Vec<TimeEntry>,
}

#[derive(Debug, Clone)]
pub enum Message {
    Test,
}

impl TimeEntriesPage {
    pub fn view(&self) -> cosmic::Element<'_, Message> {
        cosmic::widget::text::text("Hello").into()
    }

    pub fn update(&mut self, message: Message) -> app::Task<Message> {
        match message {
            Message::Test => todo!(),
        }
    }

    pub fn new() -> Self {
        TimeEntriesPage {
            time_entries: vec![],
        }
    }
}
