use chrono::{TimeDelta, Utc};
use cosmic::{
    app,
    cosmic_config::{self, ConfigState, CosmicConfigEntry},
    cosmic_theme::Spacing,
    iced::{
        self,
        futures::SinkExt,
        stream,
        widget::{column, row},
        window, Subscription,
    },
    iced_winit::commands::popup::{destroy_popup, get_popup},
    task, theme,
    widget::{
        autosize, button, container, icon,
        segmented_button::{self, Entity},
        tab_bar, Id, Text,
    },
    Element, Task,
};
use icu::locale::Locale;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, LazyLock,
};
use tokio::time;
use tracker_integrations::{ApiId, Project, TimeEntry, TogglClient, Workspace};

use crate::{
    config::{GlobalState, GLOBAL_STATE_VERSION},
    pages::{time_entries_page, timer_page},
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
}

#[derive(Debug, Clone)]
pub enum Message {
    TogglePopup,
    TabChanged(Entity),
    Tick,
    CloseRequested(window::Id),
    TimerPage(timer_page::Message),
    TimeEntriesPage(time_entries_page::Message),
    GetExistingTracker,
    ExistingTrackerGotten(Option<TimeEntry>),
    StartTimer,
    TimerStarted(Option<TimeEntry>),
    StopTimer,
    TimerStopped,
    GetProjectsForWorkspace(Workspace),
    ProjectsGotten(Option<Vec<Project>>),
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
        return format!(
            "{}:{:02}:{:02}",
            duration.num_hours(),
            duration.num_minutes() % 60,
            duration.num_seconds() % 60
        );
    } else {
        return format!(
            "{:02}:{:02}",
            duration.num_minutes() % 60,
            duration.num_seconds() % 60
        );
    }
}

impl AppletModel {
    fn horizontal_layout(&self) -> Element<'_, Message> {
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

        let state_handler = cosmic::cosmic_config::Config::new(Self::APP_ID, GLOBAL_STATE_VERSION)
            .expect("Failed to init config.");

        let state = GlobalState::get_entry(&state_handler).unwrap_or_default();

        let (timer_page, get_workspaces_task) =
            timer_page::TimerPage::new(state.clone(), state_handler.clone());

        let get_existing_tracker_task: Task<Message> =
            cosmic::task::message(Message::GetExistingTracker);
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
                timer_page: timer_page,
                time_entries_page: time_entries_page::TimeEntriesPage::new(),
                state,
                state_handler,
            },
            startup_tasks,
        )
    }

    fn subscription(&self) -> Subscription<Message> {
        let ticker = Subscription::run_with_id(
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
        );
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
            Message::CloseRequested(id) => {
                if (Some(id)) == self.popup {
                    self.popup = None;
                }
                Task::none()
            }
            Message::GetExistingTracker => cosmic::task::future(async move {
                let time_entry = TogglClient::get_current_time_entry().await;
                match time_entry {
                    Ok(entry) => Message::ExistingTrackerGotten(entry),
                    Err(_) => Message::ExistingTrackerGotten(None),
                }
            }),
            Message::ExistingTrackerGotten(time_entry) => {
                if let Some(entry) = time_entry {
                    self.state
                        .set_running_time_entry(&self.state_handler, Some(entry));
                }
                Task::none()
            }
            Message::StartTimer => {
                if let Some(selected_workspace) = &self.state.selected_workspace {
                    let selected_workspace_id = selected_workspace.id.clone();
                    return cosmic::task::future(async move {
                        let time_entry =
                            TogglClient::start_new_time_entry(selected_workspace_id).await;
                        if let Ok(time_entry) = time_entry {
                            return Message::TimerStarted(time_entry);
                        }
                        Message::Tick
                    });
                }
                Task::none()
            }
            Message::TimerStarted(time_entry) => {
                if let Some(entry) = time_entry {
                    self.state
                        .set_running_time_entry(&self.state_handler, Some(entry));
                }
                Task::none()
            }
            Message::StopTimer => {
                if let Some(entry) = self.state.running_time_entry.clone() {
                    return cosmic::task::future(async move {
                        TogglClient::stop_time_entry(&entry).await;
                        Message::TimerStopped
                    });
                }
                Task::none()
            }
            Message::TimerStopped => {
                self.state.set_running_time_entry(&self.state_handler, None);
                Task::none()
            }
            Message::GetProjectsForWorkspace(workspace) => cosmic::task::future(async move {
                let projects = TogglClient::get_workspace_projects(workspace.id).await;
                match projects {
                    Ok(projects) => Message::ProjectsGotten(Some(projects)),
                    Err(_) => Message::ProjectsGotten(None),
                }
            }),
            Message::ProjectsGotten(projects) => todo!(),
            Message::StateChanged(state) => {
                tracing::info!("State changed.");
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
