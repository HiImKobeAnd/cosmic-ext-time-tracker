use cosmic::{
    app,
    cosmic_theme::Spacing,
    iced::{
        futures::SinkExt,
        stream,
        widget::{column, row},
        window, Alignment, Subscription,
    },
    iced_winit::commands::popup::{destroy_popup, get_popup},
    theme,
    widget::{
        autosize, button, container, icon, Id, Text,
    },
    Element, Task,
};
use icu::locale::Locale;
use std::{
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

pub struct AppletModel {
    core: cosmic::Core,
    popup: Option<window::Id>,
    timer_counter: SystemTime,
    timer_running: bool,
    timer_duration: Duration,
    locale: Locale,
}

#[derive(Debug, Clone)]
pub enum Message {
    TogglePopup,
    ToggleTimer,
    Tick,
    ResetTimer,
    CloseRequested(window::Id),
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
        let reset_button = button::icon(icon::from_name("object-rotate-left-symbolic"))
            .on_press(Message::ResetTimer)
            .class(cosmic::theme::Button::AppletIcon);
        let popup_toggle_button = button::icon(icon::from_name("open-menu-symbolic"))
            .on_press(Message::TogglePopup)
            .class(cosmic::theme::Button::AppletIcon);
        Element::from(
            row!(counter, reset_button, popup_toggle_button,)
                // .align_y(Alignment::Center)
                .padding([0, self.core.applet.suggested_padding(true)]),
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
        (
            Self {
                core,
                popup: None,
                timer_counter: SystemTime::now(),
                timer_running: true,
                timer_duration: Duration::new(0, 0),
                locale: get_system_locale(),
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
        }
    }
    fn view(&self) -> cosmic::Element<'_, Self::Message> {
        autosize::autosize(self.horizontal_layout(), AUTOSIZE_MAIN_ID.clone()).into()
    }

    fn view_window(&self, id: window::Id) -> Element<'_, Self::Message> {
        let Spacing {
              ..
        } = theme::active().cosmic().spacing;

        let icon = if self.timer_running {
            icon::from_name("media-playback-stop-symbolic")
        } else {
            icon::from_name("media-playback-start-symbolic")
        };
        let counter = button::icon(icon)
            .on_press(Message::ToggleTimer)
            .class(cosmic::theme::Button::AppletIcon);
        let content = column![row![counter].align_y(Alignment::Center)];

        // let content = self.page.view();

        self.core.applet.popup_container(container(content)).into()
    }

    fn on_close_requested(&self, id: window::Id) -> Option<Self::Message> {
        Some(Message::CloseRequested(id))
    }
}
