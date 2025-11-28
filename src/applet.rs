use cosmic::{
    app,
    cosmic_theme::Spacing,
    iced::{
        futures::SinkExt,
        stream,
        widget::{column, row, vertical_space},
        window, Alignment, Rectangle, Subscription,
    },
    iced_winit::commands::popup::{destroy_popup, get_popup},
    theme,
    widget::{
        autosize, button, container, rectangle_tracker::RectangleUpdate, Id, RectangleTracker, Text,
    },
    Element, Task,
};
use icu::locale::Locale;
use std::{
    sync::LazyLock,
    time::{Duration, SystemTime},
};
use tokio::time;

use crate::fl;

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
    selected_tag: usize,
    rectangle_tracker: Option<RectangleTracker<u32>>,
    rectangle: Rectangle,
}

#[derive(Debug, Clone)]
pub enum Message {
    TogglePopup,
    Rectangle(RectangleUpdate<u32>),
    ToggleTimer,
    Tick,
    ResetTimer,
    // SetTimer(i32),
}

impl AppletModel {
    fn horizontal_layout(&self) -> Element<'_, Message> {
        let counter = button::custom(Text::new(format!("{:?}", self.timer_duration.as_secs())))
            .on_press(Message::ToggleTimer);
        let reset_button =
            button::custom(Text::new(format!("Reset"))).on_press(Message::ResetTimer);
        let popup_toggle_button =
            button::custom(Text::new(format!("Popup!"))).on_press(Message::TogglePopup);
        Element::from(
            row!(counter, reset_button, popup_toggle_button)
                .align_y(Alignment::Center)
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
                rectangle_tracker: None,
                rectangle: Rectangle::default(),
                timer_counter: SystemTime::now(),
                timer_running: true,
                timer_duration: Duration::new(0, 0),
                selected_tag: 0,
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
                    match self.timer_counter.elapsed() {
                        Ok(elapsed) => self.timer_duration += elapsed,
                        Err(_) => (),
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

                    let mut popup_settings = self.core.applet.get_popup_settings(
                        self.core.main_window_id().unwrap(),
                        new_id,
                        None,
                        None,
                        None,
                    );
                    let Rectangle {
                        x,
                        y,
                        width,
                        height,
                    } = self.rectangle;
                    popup_settings.positioner.anchor_rect = Rectangle::<i32> {
                        x: x.max(1.) as i32,
                        y: y.max(1.) as i32,
                        width: width.max(1.) as i32,
                        height: height.max(1.) as i32,
                    };

                    popup_settings.positioner.size = None;

                    get_popup(popup_settings)
                }
            }
            Message::Rectangle(u) => {
                match u {
                    RectangleUpdate::Rectangle(r) => {
                        self.rectangle = r.1;
                    }
                    RectangleUpdate::Init(tracker) => {
                        self.rectangle_tracker = Some(tracker);
                    }
                }
                Task::none()
            }
        }
    }
    fn view(&self) -> cosmic::Element<Self::Message> {
        let layout = self.horizontal_layout();
        autosize::autosize(
            if let Some(tracker) = self.rectangle_tracker.as_ref() {
                Element::from(tracker.container(0, layout).ignore_bounds(true))
            } else {
                self.horizontal_layout()
            },
            AUTOSIZE_MAIN_ID.clone().into(),
        )
        .into()
    }

    fn view_window(&self, id: window::Id) -> Element<'_, Self::Message> {
        let Spacing {
            space_xxs, space_s, ..
        } = theme::active().cosmic().spacing;
        let counter = button::custom(Text::new(format!("hello"))).on_press(Message::ToggleTimer);
        let content =
            column![row![counter].align_y(Alignment::Center).padding([12, 20])].padding([8, 0]);
        self.core.applet.popup_container(container(content)).into()
    }
}
