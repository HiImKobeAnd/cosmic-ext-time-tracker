// SPDX-License-Identifier: MPL-2.0

mod applet;
mod config;
mod i18n;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> cosmic::iced::Result {
    // Get the system's preferred languages.
    let requested_languages = i18n_embed::DesktopLanguageRequester::requested_languages();

    // Enable localizations to be applied.
    i18n::init(&requested_languages);

    tracing_subscriber::fmt::init();
    let _ = tracing_log::LogTracer::init();

    tracing::info!("Staring time tracker applet with version {VERSION}");

    // Starts the application's event loop with `()` as the application's flags.
    // cosmic::app::run::<app::AppModel>(settings, ())
    cosmic::applet::run::<applet::AppletModel>(())
}
