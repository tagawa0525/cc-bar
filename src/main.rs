mod app;
mod config;
mod data;
mod message;
mod watcher;

fn main() -> cosmic::iced::Result {
    cosmic::applet::run::<app::CcBar>(())
}
