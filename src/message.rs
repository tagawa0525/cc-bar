#[derive(Debug, Clone)]
pub enum Message {
    // ファイル監視イベント
    SessionUpdate(String), // session_id
    // タイマーイベント
    Tick,
    // ポップアップ制御
    TogglePopup,
}
