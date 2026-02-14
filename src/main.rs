mod app;
mod chart;
mod config;
mod data;
mod message;
mod watcher;

macro_rules! log {
    ($($arg:tt)*) => {{
        use std::io::Write as _;
        let msg = format!("{}\n", format_args!($($arg)*));
        let mut wrote = false;

        // CC_BAR_LOG_FILE が指定されていればそちらに書き込む
        if let Ok(path) = std::env::var("CC_BAR_LOG_FILE") {
            if let Ok(mut f) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)
            {
                let _ = f.write_all(msg.as_bytes());
                wrote = true;
            }
        } else {
            // XDG_RUNTIME_DIR 配下に書き込む（tmpfs なのでリブートで消える）
            if let Ok(dir) = std::env::var("XDG_RUNTIME_DIR") {
                let path = std::path::PathBuf::from(dir).join("cc-bar.log");
                if let Ok(mut f) = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(path)
                {
                    let _ = f.write_all(msg.as_bytes());
                    wrote = true;
                }
            }
        }

        // フォールバック: stderr に出力（journald がキャプチャ）
        if !wrote {
            let _ = std::io::stderr().write_all(msg.as_bytes());
        }
    }};
}
pub(crate) use log;

fn main() -> cosmic::iced::Result {
    log!("cc-bar: main() started");
    cosmic::applet::run::<app::CcBar>(())
}
