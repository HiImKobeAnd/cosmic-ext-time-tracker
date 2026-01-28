use cosmic::app;

use crate::{applet, config::GlobalState};

pub struct SettingsPage {
    pub state: GlobalState,
    pub state_handler: cosmic::cosmic_config::Config,
}
//
// #[derive(Debug, Clone)]
// pub enum Message {
//     ChangeTracker,
// }
//
// impl From<Message> for applet::Message {
//     fn from(message: Message) -> Self {
//         applet::Message::SettingsPage(message)
//     }
// }

// impl SettingsPage {
//     pub fn view(&self) -> cosmic::Element<'_, Message> {}
//
//     pub fn update(&mut self, message: Message) -> app::Task<Message> {
//         match message {
//             Message::ChangeTracker => self.state.set_selected_tracker(),
//         }
//     }
//
//     pub fn new(state: GlobalState, state_handler: cosmic::cosmic_config::Config) -> Self {
//         SettingsPage {
//             state,
//             state_handler,
//         }
//     }
// }
