use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Claude Code Status Line JSON
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusLineData {
    pub context_window: ContextWindow,
    pub model: Model,
    pub session_id: String,
    pub cost: Cost,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextWindow {
    pub used_percentage: u32,
    pub remaining_percentage: u32,
    pub context_window_size: u32,
    pub total_input_tokens: u32,
    pub total_output_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Model {
    pub id: String,
    pub display_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cost {
    pub total_cost_usd: f64,
    pub total_duration_ms: u64,
}

/// セッション情報
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SessionData {
    pub session_id: String,
    pub project_dir: String,
    pub model_name: String,
    pub context_used_percent: u32,
    pub cost_usd: f64,
    pub duration_ms: u64,
    pub peak_usage_percent: u32,
    pub subagent_completed_count: u32,
    pub last_updated_at: u64,
}

/// セッションストア
pub struct SessionStore {
    sessions: HashMap<String, SessionData>,
}

#[allow(dead_code)]
impl SessionStore {
    pub fn new() -> Self {
        SessionStore {
            sessions: HashMap::new(),
        }
    }

    /// Status Line JSONからセッション情報を追加/更新
    pub fn update_from_status_line(&mut self, status: StatusLineData, project_dir: String) {
        let session_id = status.session_id.clone();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let session = self
            .sessions
            .entry(session_id)
            .or_insert_with(|| SessionData {
                session_id: status.session_id.clone(),
                project_dir: project_dir.clone(),
                model_name: status.model.display_name.clone(),
                context_used_percent: status.context_window.used_percentage,
                cost_usd: status.cost.total_cost_usd,
                duration_ms: status.cost.total_duration_ms,
                peak_usage_percent: status.context_window.used_percentage,
                subagent_completed_count: 0,
                last_updated_at: now,
            });

        // Update fields
        session.context_used_percent = status.context_window.used_percentage;
        session.cost_usd = status.cost.total_cost_usd;
        session.duration_ms = status.cost.total_duration_ms;
        session.last_updated_at = now;

        // Track peak usage
        if status.context_window.used_percentage > session.peak_usage_percent {
            session.peak_usage_percent = status.context_window.used_percentage;
        }
    }

    /// セッションを追加/更新する（JSON直接）
    pub fn update_session(&mut self, session: SessionData) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        self.sessions
            .entry(session.session_id.clone())
            .and_modify(|s| {
                s.context_used_percent = session.context_used_percent;
                s.cost_usd = session.cost_usd;
                s.duration_ms = session.duration_ms;
                s.last_updated_at = now;
                if session.context_used_percent > s.peak_usage_percent {
                    s.peak_usage_percent = session.context_used_percent;
                }
            })
            .or_insert(SessionData {
                last_updated_at: now,
                ..session
            });
    }

    /// Staleセッション（5分以上更新がない）を除去
    pub fn remove_stale_sessions(&mut self, stale_threshold_secs: u64) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        self.sessions
            .retain(|_, session| now - session.last_updated_at < stale_threshold_secs);
    }

    /// すべてのセッションを取得
    pub fn get_all_sessions(&self) -> Vec<SessionData> {
        self.sessions.values().cloned().collect()
    }

    /// セッション数を取得
    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }

    /// セッションを削除
    pub fn remove_session(&mut self, session_id: &str) {
        self.sessions.remove(session_id);
    }
}

impl Default for SessionStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_line_deserialization() {
        let json = r#"{
            "context_window": {
                "used_percentage": 62,
                "remaining_percentage": 38,
                "context_window_size": 200000,
                "total_input_tokens": 124000,
                "total_output_tokens": 4500
            },
            "model": {
                "id": "claude-opus-4-6",
                "display_name": "Opus"
            },
            "session_id": "abc123xyz",
            "cost": {
                "total_cost_usd": 0.12,
                "total_duration_ms": 323000
            }
        }"#;

        let status: StatusLineData = serde_json::from_str(json).expect("Failed to parse JSON");
        assert_eq!(status.session_id, "abc123xyz");
        assert_eq!(status.context_window.used_percentage, 62);
        assert_eq!(status.model.display_name, "Opus");
    }

    #[test]
    fn test_session_store_update() {
        let mut store = SessionStore::new();

        let status = StatusLineData {
            context_window: ContextWindow {
                used_percentage: 50,
                remaining_percentage: 50,
                context_window_size: 200000,
                total_input_tokens: 100000,
                total_output_tokens: 5000,
            },
            model: Model {
                id: "claude-opus-4-6".to_string(),
                display_name: "Opus".to_string(),
            },
            session_id: "session1".to_string(),
            cost: Cost {
                total_cost_usd: 0.10,
                total_duration_ms: 100000,
            },
        };

        store.update_from_status_line(status, "/home/user/project".to_string());
        assert_eq!(store.session_count(), 1);

        let sessions = store.get_all_sessions();
        assert_eq!(sessions[0].session_id, "session1");
        assert_eq!(sessions[0].context_used_percent, 50);
        assert_eq!(sessions[0].peak_usage_percent, 50);
    }

    #[test]
    fn test_peak_usage_tracking() {
        let mut store = SessionStore::new();

        let status1 = StatusLineData {
            context_window: ContextWindow {
                used_percentage: 30,
                remaining_percentage: 70,
                context_window_size: 200000,
                total_input_tokens: 60000,
                total_output_tokens: 3000,
            },
            model: Model {
                id: "claude-opus-4-6".to_string(),
                display_name: "Opus".to_string(),
            },
            session_id: "session1".to_string(),
            cost: Cost {
                total_cost_usd: 0.05,
                total_duration_ms: 50000,
            },
        };

        store.update_from_status_line(status1, "/home/user/project".to_string());
        let sessions = store.get_all_sessions();
        assert_eq!(sessions[0].peak_usage_percent, 30);

        // Update with higher usage
        let status2 = StatusLineData {
            context_window: ContextWindow {
                used_percentage: 75,
                remaining_percentage: 25,
                context_window_size: 200000,
                total_input_tokens: 150000,
                total_output_tokens: 5000,
            },
            model: Model {
                id: "claude-opus-4-6".to_string(),
                display_name: "Opus".to_string(),
            },
            session_id: "session1".to_string(),
            cost: Cost {
                total_cost_usd: 0.15,
                total_duration_ms: 100000,
            },
        };

        store.update_from_status_line(status2, "/home/user/project".to_string());
        let sessions = store.get_all_sessions();
        assert_eq!(sessions[0].peak_usage_percent, 75);
        assert_eq!(sessions[0].context_used_percent, 75);
    }
}
