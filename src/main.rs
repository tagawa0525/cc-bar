mod app;
mod config;
mod message;

fn main() -> cosmic::iced::Result {
    cosmic::applet::run::<app::CcBar>(())
}
