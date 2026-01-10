use chrono::{DateTime, Local};
use cosmic::{
    app,
    cosmic_theme::Spacing,
    iced::{
        futures::SinkExt,
        stream,
        widget::{column, row},
        window, Length, Subscription,
    },
    iced_winit::commands::popup::{destroy_popup, get_popup},
    task, theme,
    widget::{
        self, autosize, button, container, dropdown, icon, nav_bar,
        segmented_button::{self, Entity, StyleSheet},
        tab_bar, text, text_input, Id, Text,
    },
    Apply, Element, Task,
};
use icu::locale::Locale;
use std::{
    ops::Deref,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, LazyLock,
    },
    time::{Duration, SystemTime},
};
use tokio::time;
use tracker_integrations::{TimeEntry, TogglClient};

use crate::pages::{time_entries_page, timer_page};

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

pub struct AppletModel {
    core: cosmic::Core,
    tab_model: segmented_button::SingleSelectModel,
    popup: Option<window::Id>,
    popup_page: Page,
    timer_counter: SystemTime,
    timer_running: Arc<AtomicBool>,
    timer_duration: Duration,
    locale: Locale,
    timer_page: timer_page::TimerPage,
    time_entries_page: time_entries_page::TimeEntriesPage,
}

#[derive(Debug, Clone)]
pub enum Message {
    TogglePopup,
    TabChanged(Entity),
    ToggleTimer,
    ResetTimer,
    Tick,
    CloseRequested(window::Id),
    TimerPage(timer_page::Message),
    TimeEntriesPage(time_entries_page::Message),
    GetExistingTracker,
    ExistingTrackerGotten(Option<TimeEntry>),
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

fn format_duration(duration: &Duration) -> String {
    let total_seconds = duration.as_secs();
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    match (hours, minutes, seconds) {
        (h, m, s) if h > 0 => format!("{h}:{m:02}:{s:02}"),
        _ => format!("{minutes:02}:{seconds:02}"),
    }
}

impl AppletModel {
    fn horizontal_layout(&self) -> Element<'_, Message> {
        let counter = button::custom(Text::new(format_duration(&self.timer_duration)))
            .on_press(Message::ToggleTimer)
            .class(cosmic::theme::Button::AppletIcon);
        let popup_toggle_button = button::icon(icon::from_name("open-menu-symbolic"))
            .on_press(Message::TogglePopup)
            .class(cosmic::theme::Button::AppletIcon);
        Element::from(
            row!(counter, popup_toggle_button,), // .align_y(Alignment::Center)
                                                 // .padding([0, self.core.applet.suggested_padding(true)]),
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

        let timer_running = Arc::new(AtomicBool::new(true));

        (
            Self {
                core,
                popup: None,
                timer_counter: SystemTime::now(),
                timer_running: Arc::clone(&timer_running),
                timer_duration: Duration::new(0, 0),
                locale: get_system_locale(),
                popup_page: Page::Timer,
                tab_model,
                timer_page: timer_page::TimerPage::new(Arc::clone(&timer_running)),
                time_entries_page: time_entries_page::TimeEntriesPage::new(),
            },
            Task::none(),
        )
    }

    fn subscription(&self) -> Subscription<Message> {
        fn time_ticker() -> Subscription<Message> {
            Subscription::run_with_id(
                "ticker",
                stream::channel(1, |mut output| async move {
                    let period = 1;
                    let mut timer = time::interval(time::Duration::from_secs(period));
                    timer.set_missed_tick_behavior(time::MissedTickBehavior::Skip);

                    loop {
                        tokio::select! {
                            _ = timer.tick() => {
                                let _ = output.send(Message::Tick).await;
                            }
                        }
                    }
                }),
            )
        }
        time_ticker()
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
            Message::ToggleTimer => {
                if self.timer_running.load(Ordering::Relaxed) {
                    self.timer_running.store(false, Ordering::Relaxed);
                } else {
                    self.timer_running.store(true, Ordering::Relaxed);
                    self.timer_counter = SystemTime::now()
                }
                Task::none()
            }
            Message::ResetTimer => {
                self.timer_duration = Duration::new(0, 0);
                self.timer_counter = SystemTime::now();
                Task::none()
            }
            Message::Tick => {
                if self.timer_running.load(Ordering::Relaxed) {
                    if let Ok(elapsed) = self.timer_counter.elapsed() {
                        self.timer_duration += elapsed
                    }
                    self.timer_counter = SystemTime::now();
                }
                Task::none()
            }
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
            Message::CloseRequested(id) => {
                if (Some(id)) == self.popup {
                    self.popup = None;
                }
                Task::none()
            }
            Message::GetExistingTracker => cosmic::task::future(async {
                Message::ExistingTrackerGotten(
                    tokio::spawn({
                        let client = TogglClient::new();
                        let current_time_entry = client.get_current_time_entry().await;
                    })
                    .await,
                )
            }),
            Message::ExistingTrackerGotten(entry) => match entry {
                Some(_) => todo!(),
                None => todo!(),
            },
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
            Page::Settings => todo!(),
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
