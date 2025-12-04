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
    theme,
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
    sync::LazyLock,
    time::{Duration, SystemTime},
};
use tokio::time;

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

pub struct Project {
    id: String,
    name: String,
}
pub struct Tag {
    id: String,
    name: String,
}
pub struct TrackingEntry {
    id: String,
    duration: Duration,
    start_time: DateTime<Local>, // !TODO Research what implications that using local will have
    stop_time: DateTime<Local>,  // !TODO Research what implications that using local will have
    project_id: String,
    tag_ids: Vec<String>,
}

pub struct AppletModel {
    core: cosmic::Core,
    tab_model: segmented_button::SingleSelectModel,
    popup: Option<window::Id>,
    popup_page: Page,
    timer_counter: SystemTime,
    timer_running: bool,
    timer_duration: Duration,
    locale: Locale,
    current_task: String,
    current_tag: Option<usize>,
    tag_selections: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum Message {
    TogglePopup,
    TabChanged(Entity),
    ToggleTimer,
    ResetTimer,
    Tick,
    TaskTextChanged(String),
    TagChanged(usize),
    CloseRequested(window::Id),
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
            row!(counter, popup_toggle_button,)
                // .align_y(Alignment::Center)
                .padding([0, self.core.applet.suggested_padding(true)]),
        )
        // .explain(cosmic::iced::Color::WHITE)
    }

    fn timer_page(&self) -> Element<'_, Message> {
        let tab_bar = tab_bar::horizontal(&self.tab_model).on_activate(Message::TabChanged);

        let task_selector = text_input::text_input("Task", self.current_task.clone())
            .on_input(Message::TaskTextChanged);

        let tag_selector = dropdown::dropdown(
            self.tag_selections.clone(),
            self.current_tag,
            Message::TagChanged,
        );

        let timer = text::text(format_duration(&self.timer_duration));

        let toggle_timer_button = button::icon(if self.timer_running {
            icon::from_name("media-playback-stop-symbolic")
        } else {
            icon::from_name("media-playback-start-symbolic")
        })
        .on_press(Message::ToggleTimer)
        .class(cosmic::theme::Button::AppletIcon);

        let reset_button = button::icon(icon::from_name("object-rotate-left-symbolic"))
            .on_press(Message::ResetTimer)
            .class(cosmic::theme::Button::AppletIcon);

        let tab_bar_element = Element::from(tab_bar);

        Element::from(column![
            tab_bar_element,
            task_selector.width(Length::Fill),
            tag_selector.width(Length::Fill),
            row![timer, toggle_timer_button, reset_button]
        ])
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

        (
            Self {
                core,
                popup: None,
                timer_counter: SystemTime::now(),
                timer_running: true,
                timer_duration: Duration::new(0, 0),
                locale: get_system_locale(),
                popup_page: Page::Timer,
                current_task: String::new(),
                current_tag: None,
                tag_selections: vec![
                    "Systemudvikling".to_string(),
                    "Programmering".to_string(),
                    "Teknologi".to_string(),
                ],
                tab_model,
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
            Message::ToggleTimer => {
                if self.timer_running {
                    self.timer_running = false;
                } else {
                    self.timer_running = true;
                    self.timer_counter = SystemTime::now()
                }
                Task::none()
            }
            Message::Tick => {
                if self.timer_running {
                    if let Ok(elapsed) = self.timer_counter.elapsed() {
                        self.timer_duration += elapsed
                    }
                    self.timer_counter = SystemTime::now();
                }
                Task::none()
            }
            Message::ResetTimer => {
                self.timer_duration = Duration::new(0, 0);
                self.timer_counter = SystemTime::now();
                Task::none()
            }
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
            Message::CloseRequested(id) => {
                if (Some(id)) == self.popup {
                    self.popup = None;
                }
                Task::none()
            }
            Message::TaskTextChanged(task) => {
                self.current_task = task;
                Task::none()
            }
            Message::TagChanged(tag) => {
                self.current_tag = Some(tag);
                Task::none()
            }
            Message::TabChanged(entity) => {
                self.tab_model.activate(entity);
                if let Some(page) = self.tab_model.data::<Page>(entity) {
                    self.popup_page = page.clone();
                }
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
            Page::Timer => self.timer_page(),
            Page::Log => todo!(),
            Page::Settings => todo!(),
        };
        self.core
            .applet
            .popup_container(container(content))
            .min_height(200.) // !HACK Fix for dropdown getting cut off
            .min_width(200.) // !HACK Fix for dropdown getting cut off
            .into()
    }

    fn on_close_requested(&self, id: window::Id) -> Option<Self::Message> {
        Some(Message::CloseRequested(id))
    }

    // fn nav_model(&self) -> Option<&cosmic::widget::nav_bar::Model> {
    // Some(&self.nav)
    // }

    // fn on_nav_select(&mut self, id: cosmic::widget::nav_bar::Id) -> app::Task<Self::Message> {
    // self.nav.activate(id);
    // Task::none()
    // }
}
