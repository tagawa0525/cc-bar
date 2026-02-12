mod app;
mod chart;
mod config;
mod data;
mod message;
mod watcher;

macro_rules! log {
    ($($arg:tt)*) => {{
        use std::io::Write as _;
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("/tmp/cc-bar.log")
        {
            let _ = writeln!(f, "{}", format_args!($($arg)*));
        }
    }};
}
pub(crate) use log;

fn main() -> cosmic::iced::Result {
    log!("cc-bar: main() started");
    cosmic::applet::run::<app::CcBar>(())
}
