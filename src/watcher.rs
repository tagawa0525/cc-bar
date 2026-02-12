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
            crate::log!(
                "cc-bar: watcher started, sessions_dir={}",
                sessions_dir.display()
            );

            // 起動時に既存のセッションファイルを読み込む
            if let Ok(mut entries) = tokio::fs::read_dir(&sessions_dir).await {
                while let Ok(Some(entry)) = entries.next_entry().await {
                    if let Some(filename) = entry.file_name().to_str() {
                        if let Some(session_id) = filename.strip_suffix(".json") {
                            let _ = output
                                .send(Message::SessionUpdate(session_id.to_string()))
                                .await;
                        }
                    }
                }
            }

            let inotify = match Inotify::init() {
                Ok(i) => {
                    crate::log!("cc-bar: inotify initialized");
                    i
                }
                Err(e) => {
                    crate::log!("cc-bar: inotify init failed: {}", e);
                    std::future::pending::<()>().await;
                    unreachable!()
                }
            };

            // MOVED_TO: relay.shのアトミックリネーム(mv tmp → .json)を検知
            // CLOSE_WRITE: 直接書き込みのフォールバック
            // CREATE: 新規ファイル作成
            // DELETE: ファイル削除でセッション除去
            if inotify
                .watches()
                .add(
                    &sessions_dir,
                    WatchMask::MOVED_TO
                        | WatchMask::CLOSE_WRITE
                        | WatchMask::CREATE
                        | WatchMask::DELETE,
                )
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

                if let Some(name) = event.name {
                    if let Some(filename) = name.to_str() {
                        if let Some(session_id) = filename.strip_suffix(".json") {
                            let msg = if event.mask.contains(EventMask::DELETE) {
                                Message::SessionRemoved(session_id.to_string())
                            } else if event.mask.contains(EventMask::MOVED_TO)
                                || event.mask.contains(EventMask::CLOSE_WRITE)
                                || event.mask.contains(EventMask::CREATE)
                            {
                                Message::SessionUpdate(session_id.to_string())
                            } else {
                                continue;
                            };
                            let _ = output.send(msg).await;
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
            let mut interval = interval(std::time::Duration::from_secs(10));
            loop {
                interval.tick().await;
                let _ = output.send(Message::Tick).await;
            }
        })
    })
}
