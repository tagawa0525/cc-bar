use crate::message::Message;
use cosmic::iced::{futures::SinkExt, stream, Subscription};
use futures_util::StreamExt;
use inotify::{EventMask, Inotify, WatchMask};
use std::path::PathBuf;
use tokio::time::interval;

/// セッションディレクトリのファイル監視Subscription
/// inotifyのasync EventStream APIを使用
pub fn watch_sessions() -> Subscription<Message> {
    Subscription::run(|| {
        stream::channel(100, |mut output| async move {
            let runtime_dir = dirs::runtime_dir().unwrap_or_else(|| {
                let uid = unsafe { libc::getuid() };
                PathBuf::from(format!("/run/user/{}", uid))
            });
            let sessions_dir = runtime_dir.join("cc-bar").join("sessions");

            // ディレクトリが存在しない場合は作成
            let _ = tokio::fs::create_dir_all(&sessions_dir).await;

            let inotify = match Inotify::init() {
                Ok(i) => i,
                Err(_) => {
                    // inotify初期化失敗時は無限待機（フォールバックなし）
                    std::future::pending::<()>().await;
                    unreachable!()
                }
            };

            if inotify
                .watches()
                .add(&sessions_dir, WatchMask::CREATE | WatchMask::CLOSE_WRITE)
                .is_err()
            {
                std::future::pending::<()>().await;
                unreachable!()
            }

            // async EventStreamを使用（blocking APIの代わり）
            let mut buffer = [0u8; 4096];
            let mut event_stream = match inotify.into_event_stream(&mut buffer) {
                Ok(s) => s,
                Err(_) => {
                    std::future::pending::<()>().await;
                    unreachable!()
                }
            };

            while let Some(event_or_error) = event_stream.next().await {
                let event = match event_or_error {
                    Ok(e) => e,
                    Err(_) => continue,
                };

                // CLOSE_WRITEまたはCREATEの.jsonファイルのみ処理
                if !(event.mask.contains(EventMask::CLOSE_WRITE)
                    || event.mask.contains(EventMask::CREATE))
                {
                    continue;
                }

                if let Some(name) = event.name {
                    if let Some(filename) = name.to_str() {
                        if let Some(session_id) = filename.strip_suffix(".json") {
                            let _ = output
                                .send(Message::SessionUpdate(session_id.to_string()))
                                .await;
                        }
                    }
                }
            }
        })
    })
}

/// 定期タイマー Subscription (stale除去用)
pub fn tick_timer() -> Subscription<Message> {
    Subscription::run(|| {
        stream::channel(100, |mut output| async move {
            let mut interval = interval(std::time::Duration::from_secs(30));
            loop {
                interval.tick().await;
                let _ = output.send(Message::Tick).await;
            }
        })
    })
}
