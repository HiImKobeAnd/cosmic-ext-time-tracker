use cosmic::{
    app,
    dbus_activation::subscription,
    iced::{futures::SinkExt, stream, Subscription},
    iced_widget::row,
    widget::button,
    Element, Task,
};
use std::time::{Duration, SystemTime};

pub struct AppletModel {
    core: cosmic::Core,
    timer_counter: SystemTime,
    timer_running: bool,
    timer_duration: Duration,
}

#[derive(Debug, Clone)]
pub enum Message {
    // TogglePopup,
    ToggleTimer,
    Tick,
    // ResetTimer,
    // SetTimer(i32),
}

impl AppletModel {
    fn horizontal_layout(&self) -> Element<'_, Message> {
        Element::from(row!(self
            .core
            .applet
            .text(self.timer_duration.as_secs().to_string())))
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
            },
            Task::none(),
        )
    }

    fn view(&self) -> cosmic::Element<'_, Self::Message> {
        let button = button::custom(self.horizontal_layout()).on_press_down(Message::ToggleTimer);
        button.into()
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
        }
    }
    fn subscription(&self) -> Subscription<Message> {
        fn time_ticker() -> Subscription<Message> {
            Subscription::run_with_id(
                "ticker",
                stream::channel(1, |mut output| async move {
                    loop {
                        let _ = output.send(Message::Tick).await;
                    }
                }),
            )
        }
        time_ticker()
    }
}
