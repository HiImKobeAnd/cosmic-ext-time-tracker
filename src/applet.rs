use cosmic::{
    app,
    iced::{futures::SinkExt, stream, Alignment, Length, Padding, Subscription},
    iced_widget::row,
    widget::{autosize, button, container, dropdown, vertical_space, Id, Text},
    Element, Task,
};
use std::{
    borrow::Cow,
    collections::{hash_map, HashMap, HashSet},
    sync::LazyLock,
    time::{Duration, SystemTime},
    usize,
};
use tokio::time;

use crate::fl;

static AUTOSIZE_MAIN_ID: LazyLock<Id> = LazyLock::new(|| Id::new("autosize-main"));

pub struct AppletModel {
    core: cosmic::Core,
    timer_counter: SystemTime,
    timer_running: bool,
    timer_duration: Duration,
    selected_tag: usize,
}

#[derive(Debug, Clone)]
pub enum Message {
    ToggleTimer,
    Tick,
    ResetTimer,
    // SetTimer(i32),
    SelectedTag(usize),
}

impl AppletModel {
    fn horizontal_layout(&self) -> Element<'_, Message> {
        let counter = button::custom(Text::new(format!("{:?}", self.timer_duration.as_secs())))
            .on_press(Message::ToggleTimer);
        let reset_button =
            button::custom(Text::new(format!("Reset"))).on_press(Message::ResetTimer);
        Element::from(
            row!(counter, reset_button)
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
    fn init(core: app::Core, _flags: Self::Flags) -> (Self, app::Task<self::Message>) {
        (
            Self {
                core,
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
                    let mut period = 1;
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
                if (self.timer_running) {
                    self.timer_running = false;
                } else {
                    self.timer_running = true;
                    self.timer_counter = SystemTime::now()
                }
                Task::none()
            }
            Message::Tick => {
                if (self.timer_running) {
                    match self.timer_counter.elapsed() {
                        Ok(elapsed) => self.timer_duration += elapsed,
                        Err(_) => (),
                    }
                    self.timer_counter = SystemTime::now();
                }
                Task::none()
            }
            Message::SelectedTag(_) => {
                println!("Changed");
                Task::none()
            }
            Message::ResetTimer => {
                self.timer_duration = Duration::new(0, 0);
                self.timer_counter = SystemTime::now();
                Task::none()
            }
        }
    }
    fn view(&self) -> cosmic::Element<Self::Message> {
        autosize::autosize(self.horizontal_layout(), AUTOSIZE_MAIN_ID.clone()).into()
    }

    fn style(&self) -> Option<cosmic::iced_runtime::Appearance> {
        Some(cosmic::applet::style())
    }
}
