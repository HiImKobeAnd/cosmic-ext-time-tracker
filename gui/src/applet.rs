use chrono::{TimeDelta, Utc};
use cosmic::{
    app,
    cosmic_config::CosmicConfigEntry,
    cosmic_theme::Spacing,
    iced::{
        border,
        widget::{column, row},
        window, Alignment, Color, Length, Subscription,
    },
    iced_winit::commands::popup::{destroy_popup, get_popup},
    theme,
    widget::{
        autosize, button, container, icon,
        segmented_button::{self, Entity},
        tab_bar, Id, Text,
    },
    Element, Task,
};
use icu::locale::Locale;
use std::{
    sync::{Arc, LazyLock},
    time::Duration,
};
use tracker_integrations::{
    get_api_key, Authenticated, Integration, TimeEntry, TogglClient, Unauthenticated,
};

use crate::{
    config::{GlobalState, GLOBAL_STATE_VERSION},
    pages::{
        settings_page::{self, SettingsPage},
        time_entries_page,
        timer_page::{self},
    },
};

static AUTOSIZE_MAIN_ID: LazyLock<Id> = LazyLock::new(|| Id::new("autosize-main"));

fn get_system_locale() -> Locale {
    for var in ["LC_TIME", "LC_ALL", "LANG"] {
        if let Ok(locale_str) = std::env::var(var) {
            let cleaned_locale = locale_str
                .split('.')
                .next()
                .unwrap_or(&locale_str)
                .replace('_', "-");

            if let Ok(locale) = Locale::try_from_str(&cleaned_locale) {
                return locale;
            }

            if let Some(lang) = cleaned_locale.split('-').next() {
                if let Ok(locale) = Locale::try_from_str(lang) {
                    return locale;
                }
            }
        }
    }
    tracing::warn!("No valid locale found in environment, using fallback");
    Locale::try_from_str("en-US").expect("Failed to parse fallback locale 'en-US'")
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum SubscriptionId {
    StateWatch,
}

pub enum Authentication {
    Auth(TogglClient<Authenticated>),
    UnAuth(TogglClient<Unauthenticated>),
}

impl Authentication {
    pub fn as_authenticated(&self) -> Option<TogglClient<Authenticated>> {
        match self {
            Authentication::Auth(client) => Some(client.clone()),
            _ => None,
        }
    }
}

pub struct AppletModel {
    core: cosmic::Core,
    state: GlobalState,
    state_handler: cosmic::cosmic_config::Config,
    locale: Locale,
    tab_model: segmented_button::SingleSelectModel,
    popup: Option<window::Id>,
    popup_page: Page,
    timer_page: timer_page::TimerPage,
    time_entries_page: time_entries_page::TimeEntriesPage,
    settings_page: settings_page::SettingsPage,
    integration_client: Option<Arc<Authentication>>,
}

#[derive(Debug, Clone)]
pub enum Message {
    TogglePopup,
    TabChanged(Entity),
    Tick,
    CloseRequested(window::Id),
    TimerPage(timer_page::Message),
    TimeEntriesPage(time_entries_page::Message),
    SettingsPage(settings_page::Message),
    GetExistingTimeEntry,
    ExistingTimeEntryGotten(Option<TimeEntry>),
    StartTimer,
    TimerStarted(TimeEntry),
    StopTimer,
    TimerStopped,
    StateChanged(GlobalState),
}

impl From<time_entries_page::Message> for Message {
    fn from(message: time_entries_page::Message) -> Self {
        Message::TimeEntriesPage(message)
    }
}

#[derive(Debug, Clone)]
pub enum Page {
    Timer,
    Log,
    Settings,
}

fn format_duration(duration: &TimeDelta) -> String {
    if duration.num_hours() > 0 {
        format!(
            "{}:{:02}:{:02}",
            duration.num_hours(),
            duration.num_minutes() % 60,
            duration.num_seconds() % 60
        )
    } else {
        format!(
            "{:02}:{:02}",
            duration.num_minutes() % 60,
            duration.num_seconds() % 60
        )
    }
}

fn color_circle(color: Color, size: f32) -> Element<'static, Message> {
    container("")
        .width(Length::Fixed(size))
        .height(Length::Fixed(size))
        .style(move |_theme| container::Style {
            background: Some(color.into()),
            border: border::rounded(size / 2.0),
            ..Default::default()
        })
        .into()
}

impl AppletModel {
    fn horizontal_layout(&self) -> Element<'_, Message> {
        let applet_height: f32 = self.core.applet.suggested_size(true).1.into();
        let mut project_indicator = color_circle(Color::WHITE, 32.);
        if let Some(selected_project) = self.state.selected_project.clone() {
            project_indicator = color_circle(
                Color::parse(&selected_project.color).unwrap_or(Color::WHITE),
                applet_height / 2.,
            )
        };

        let counter = match &self.state.running_time_entry {
            Some(entry) => button::custom(Text::new(format_duration(
                &Utc::now().signed_duration_since(entry.start_time),
            )))
            .on_press(Message::StopTimer)
            .class(cosmic::theme::Button::AppletIcon),
            None => button::custom(Text::new(format_duration(&TimeDelta::zero())))
                .on_press(Message::StartTimer)
                .class(cosmic::theme::Button::AppletIcon),
        };

        let popup_toggle_button = button::icon(icon::from_name("open-menu-symbolic"))
            .on_press(Message::TogglePopup)
            .class(cosmic::theme::Button::AppletIcon);
        Element::from(
            row!(project_indicator, counter, popup_toggle_button,).align_y(Alignment::Center),
        )
        // .explain(cosmic::iced::Color::WHITE)
    }
}

impl cosmic::Application for AppletModel {
    type Executor = cosmic::executor::Default;
    type Flags = ();
    type Message = Message;
    const APP_ID: &'static str = "com.github.hiimkobeand.cosmic-ext-time-tracker";

    fn core(&self) -> &cosmic::app::Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut cosmic::app::Core {
        &mut self.core
    }

    fn style(&self) -> Option<cosmic::iced_runtime::Appearance> {
        Some(cosmic::applet::style())
    }

    fn init(core: app::Core, _flags: Self::Flags) -> (Self, app::Task<self::Message>) {
        let mut tab_model = segmented_button::SingleSelectModel::default();

        tab_model
            .insert()
            .text("Timer")
            .activate()
            .data::<Page>(Page::Timer);
        tab_model.insert().text("Log").data::<Page>(Page::Log);
        tab_model
            .insert()
            .text("Settings")
            .data::<Page>(Page::Settings);

        let state_handler = cosmic::cosmic_config::Config::new(Self::APP_ID, GLOBAL_STATE_VERSION)
            .expect("Failed to init config.");

        let state = GlobalState::get_entry(&state_handler).unwrap_or_default();

        let integration_client =
            state
                .clone()
                .selected_tracker
                .map(|Integration::TogglIntegration| {
                    let client = TogglClient::new();
                    let auth = match get_api_key(&Integration::TogglIntegration) {
                        Ok(api_key) => Authentication::Auth(client.authenticate(api_key)),
                        Err(_) => Authentication::UnAuth(client),
                    };
                    Arc::new(auth)
                });

        let (timer_page, get_workspaces_task) = timer_page::TimerPage::new(
            integration_client.clone(),
            state.clone(),
            state_handler.clone(),
        );

        let settings_page = SettingsPage::new(state.clone(), state_handler.clone());

        let get_existing_tracker_task: Task<Message> =
            cosmic::task::message(Message::GetExistingTimeEntry);
        let startup_tasks = cosmic::task::batch([
            get_existing_tracker_task,
            get_workspaces_task.map(self::Message::TimerPage),
        ]);

        (
            Self {
                core,
                popup: None,
                locale: get_system_locale(),
                popup_page: Page::Timer,
                tab_model,
                timer_page,
                time_entries_page: time_entries_page::TimeEntriesPage::new(),
                settings_page,
                state,
                state_handler,
                integration_client,
            },
            startup_tasks,
        )
    }

    fn subscription(&self) -> Subscription<Message> {
        let ticker = cosmic::iced::time::every(Duration::from_secs(1)).map(|_| Message::Tick);
        let state_watcher = cosmic::cosmic_config::config_subscription(
            SubscriptionId::StateWatch,
            Self::APP_ID.into(),
            GLOBAL_STATE_VERSION,
        )
        .map(|update| Message::StateChanged(update.config));
        Subscription::batch(vec![ticker, state_watcher])
    }

    fn update(&mut self, message: Self::Message) -> app::Task<Self::Message> {
        match message {
            Message::TogglePopup => {
                if let Some(p) = self.popup.take() {
                    destroy_popup(p)
                } else {
                    let new_id = window::Id::unique();
                    self.popup = Some(new_id);

                    let popup_settings = self.core.applet.get_popup_settings(
                        self.core.main_window_id().unwrap(),
                        new_id,
                        None,
                        None,
                        None,
                    );

                    get_popup(popup_settings)
                }
            }
            Message::TabChanged(entity) => {
                self.tab_model.activate(entity);
                if let Some(page) = self.tab_model.data::<Page>(entity) {
                    self.popup_page = page.clone();
                }
                Task::none()
            }
            Message::Tick => Task::none(),
            Message::TimerPage(message) => {
                self.timer_page.update(message).map(|action| match action {
                    cosmic::Action::None => cosmic::Action::None,
                    cosmic::Action::App(m) => cosmic::Action::App(m.into()),
                    cosmic::Action::Cosmic(a) => cosmic::Action::Cosmic(a),
                    cosmic::Action::DbusActivation(message) => {
                        cosmic::Action::DbusActivation(message)
                    }
                })
            }
            Message::TimeEntriesPage(message) => {
                self.time_entries_page
                    .update(message)
                    .map(|action| match action {
                        cosmic::Action::None => cosmic::Action::None,
                        cosmic::Action::App(m) => cosmic::Action::App(m.into()),
                        cosmic::Action::Cosmic(a) => cosmic::Action::Cosmic(a),
                        cosmic::Action::DbusActivation(message) => {
                            cosmic::Action::DbusActivation(message)
                        }
                    })
            }
            Message::SettingsPage(message) => {
                self.settings_page
                    .update(message)
                    .map(|action| match action {
                        cosmic::Action::None => cosmic::Action::None,
                        cosmic::Action::App(m) => cosmic::Action::App(m.into()),
                        cosmic::Action::Cosmic(a) => cosmic::Action::Cosmic(a),
                        cosmic::Action::DbusActivation(message) => {
                            cosmic::Action::DbusActivation(message)
                        }
                    })
            }
            Message::CloseRequested(id) => {
                if (Some(id)) == self.popup {
                    self.popup = None;
                }
                Task::none()
            }
            Message::GetExistingTimeEntry => self
                .integration_client
                .as_ref()
                .and_then(|auth| auth.as_authenticated())
                .map(|client| {
                    cosmic::task::future(async move {
                        let time_entry = client.get_current_time_entry().await;
                        match time_entry {
                            Ok(entry) => Message::ExistingTimeEntryGotten(entry),
                            Err(_) => Message::ExistingTimeEntryGotten(None),
                        }
                    })
                })
                .unwrap_or(Task::none()),

            Message::ExistingTimeEntryGotten(time_entry) => {
                let _ = self
                    .state
                    .set_running_time_entry(&self.state_handler, time_entry);
                Task::none()
            }
            Message::StartTimer => {
                if let Some(selected_workspace) = &self.state.selected_workspace {
                    let selected_workspace_id = selected_workspace.id.clone();
                    let mut selected_project_id = None;
                    if let Some(selected_project) = &self.state.selected_project {
                        selected_project_id = Some(selected_project.id.clone());
                    };
                    let current_description = self.state.current_description.clone();
                    return self
                        .integration_client
                        .as_ref()
                        .and_then(|auth| auth.as_authenticated())
                        .map(|client| {
                            return cosmic::task::future(async move {
                                let time_entry = client
                                    .start_new_time_entry(
                                        selected_workspace_id,
                                        selected_project_id,
                                        current_description,
                                    )
                                    .await;
                                if let Ok(time_entry) = time_entry {
                                    return Message::TimerStarted(time_entry);
                                }
                                Message::Tick
                            });
                        })
                        .unwrap_or(Task::none());
                }
                Task::none()
            }
            Message::TimerStarted(time_entry) => {
                let _ = self
                    .state
                    .set_running_time_entry(&self.state_handler, Some(time_entry));
                Task::none()
            }
            Message::StopTimer => {
                if let Some(entry) = self.state.running_time_entry.clone() {
                    return self
                        .integration_client
                        .as_ref()
                        .and_then(|auth| auth.as_authenticated())
                        .map(|client| {
                            return cosmic::task::future(async move {
                                let _ = client.stop_time_entry(entry.workspace_id, entry.id).await;
                                Message::TimerStopped
                            });
                        })
                        .unwrap_or(Task::none());
                }
                Task::none()
            }
            Message::TimerStopped => {
                let _ = self.state.set_running_time_entry(&self.state_handler, None);
                Task::none()
            }
            Message::StateChanged(state) => {
                tracing::info!("State changed.");
                if self.state.selected_tracker != state.selected_tracker {
                    self.integration_client =
                        state
                            .clone()
                            .selected_tracker
                            .map(|Integration::TogglIntegration| {
                                let client = TogglClient::new();
                                let auth = match get_api_key(&Integration::TogglIntegration) {
                                    Ok(api_key) => {
                                        Authentication::Auth(client.authenticate(api_key))
                                    }
                                    Err(_) => Authentication::UnAuth(client),
                                };
                                Arc::new(auth)
                            });
                }
                self.state = state.clone();
                self.timer_page.state = state;
                Task::none()
            }
        }
    }
    fn view(&self) -> cosmic::Element<'_, Self::Message> {
        autosize::autosize(self.horizontal_layout(), AUTOSIZE_MAIN_ID.clone()).into()
    }

    fn view_window(&self, id: window::Id) -> Element<'_, Self::Message> {
        let Spacing { .. } = theme::active().cosmic().spacing;
        let content = match self.popup_page {
            Page::Timer => self.timer_page.view().map(Message::TimerPage),
            Page::Log => self.time_entries_page.view().map(Message::TimeEntriesPage),
            Page::Settings => self.settings_page.view().map(Message::SettingsPage),
        };

        let tab_bar = tab_bar::horizontal(&self.tab_model).on_activate(Message::TabChanged);
        let tab_bar_element = Element::from(tab_bar);

        self.core
            .applet
            .popup_container(container(column![tab_bar_element, content]))
            .min_height(200.) // !HACK Fix for dropdown getting cut off
            .min_width(200.) // !HACK Fix for dropdown getting cut off
            .into()
    }

    fn on_close_requested(&self, id: window::Id) -> Option<Self::Message> {
        Some(Message::CloseRequested(id))
    }
}
