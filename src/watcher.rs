use crate::message::Message;
use cosmic::iced::{futures::SinkExt, stream, Subscription};
use inotify::{EventMask, Inotify, WatchMask};
use std::path::PathBuf;
use std::time::Duration;
use tokio::time::interval;

/// セッションディレクトリのファイル監視Subscription
pub fn watch_sessions() -> Subscription<Message> {
    Subscription::run(|| {
        stream::channel(100, |mut output| async move {
            let runtime_dir = dirs::runtime_dir().unwrap_or_else(|| {
                PathBuf::from(format!("/run/user/{}", nix::unistd::getuid().as_raw()))
            });
            let sessions_dir = runtime_dir.join("cc-bar").join("sessions");

            // ディレクトリが存在しない場合は作成
            let _ = tokio::fs::create_dir_all(&sessions_dir).await;

            match Inotify::init() {
                Ok(mut inotify) => {
                    // ファイル追加・変更を監視
                    if let Ok(_) = inotify.watches().add(
                        &sessions_dir,
                        WatchMask::CREATE | WatchMask::MODIFY | WatchMask::CLOSE_WRITE,
                    ) {
                        let mut buffer = [0u8; 4096];

                        loop {
                            match inotify.read_events_blocking(&mut buffer) {
                                Ok(events) => {
                                    for event in events {
                                        // .json ファイルのみ処理
                                        if let Some(name) = event.name {
                                            if let Some(filename) = name.to_str() {
                                                if filename.ends_with(".json") {
                                                    if event.mask.contains(EventMask::CLOSE_WRITE) {
                                                        // ファイルが閉じられた時点で更新
                                                        let session_id = filename
                                                            .trim_end_matches(".json")
                                                            .to_string();
                                                        let _ = output
                                                            .send(Message::SessionUpdate(
                                                                session_id,
                                                            ))
                                                            .await;
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                Err(_e) => {
                                    // エラーの場合は再試行待機
                                    tokio::time::sleep(Duration::from_secs(1)).await;
                                }
                            }
                        }
                    }
                }
                Err(_e) => {
                    // inotify初期化失敗時、タイマーで監視ループをシミュレート
                    let mut tick = interval(Duration::from_secs(1));
                    loop {
                        tick.tick().await;
                        // TODO: フォールバック監視ロジック
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
            let mut interval = interval(Duration::from_secs(30));
            loop {
                interval.tick().await;
                let _ = output.send(Message::Tick).await;
            }
        })
    })
}
