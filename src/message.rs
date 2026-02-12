use cosmic::iced::window;

#[derive(Debug, Clone)]
pub enum Message {
    // ファイル監視イベント
    SessionUpdate(String), // session_id
    // タイマーイベント
    Tick,
    // ポップアップ制御
    TogglePopup,
    // ウィンドウ閉じ要求
    CloseRequested(window::Id),
}
