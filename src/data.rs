use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::time::{SystemTime, UNIX_EPOCH};

pub const MAX_HISTORY_SAMPLES: usize = 21;

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
    #[serde(default)]
    pub used_percentage: Option<u32>,
    #[serde(default)]
    pub remaining_percentage: Option<u32>,
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
    pub usage_history: VecDeque<f64>,
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

        let used = status.context_window.used_percentage.unwrap_or(0);

        let session = self
            .sessions
            .entry(session_id)
            .or_insert_with(|| SessionData {
                session_id: status.session_id.clone(),
                project_dir: project_dir.clone(),
                model_name: status.model.display_name.clone(),
                context_used_percent: used,
                cost_usd: status.cost.total_cost_usd,
                duration_ms: status.cost.total_duration_ms,
                peak_usage_percent: used,
                subagent_completed_count: 0,
                last_updated_at: now,
                usage_history: VecDeque::new(),
            });

        // Update fields
        session.model_name = status.model.display_name.clone();
        session.context_used_percent = used;
        session.cost_usd = status.cost.total_cost_usd;
        session.duration_ms = status.cost.total_duration_ms;
        session.last_updated_at = now;

        // Track peak usage
        if used > session.peak_usage_percent {
            session.peak_usage_percent = used;
        }

        // Track usage history
        session.usage_history.push_back(used as f64);
        if session.usage_history.len() > MAX_HISTORY_SAMPLES {
            session.usage_history.pop_front();
        }
    }

    /// セッションを追加/更新する（JSON直接）
    pub fn update_session(&mut self, session: SessionData) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let used = session.context_used_percent;
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
                s.usage_history.push_back(used as f64);
                if s.usage_history.len() > MAX_HISTORY_SAMPLES {
                    s.usage_history.pop_front();
                }
            })
            .or_insert_with(|| {
                let mut s = SessionData {
                    last_updated_at: now,
                    ..session
                };
                s.usage_history.push_back(used as f64);
                if s.usage_history.len() > MAX_HISTORY_SAMPLES {
                    s.usage_history.pop_front();
                }
                s
            });
    }

    /// ファイルのmtimeが古いセッションを除去
    pub fn remove_stale_by_mtime(&mut self, sessions_dir: &std::path::Path, threshold_secs: u64) {
        let now = SystemTime::now();
        self.sessions.retain(|session_id, _| {
            let path = sessions_dir.join(format!("{}.json", session_id));
            match std::fs::metadata(&path) {
                Ok(meta) => match meta.modified() {
                    Ok(mtime) => now
                        .duration_since(mtime)
                        .map(|d| d.as_secs() < threshold_secs)
                        .unwrap_or(true),
                    Err(_) => false,
                },
                Err(_) => false, // ファイルが存在しない→除去
            }
        });
    }

    /// すべてのセッションを取得（Opus→Sonnet→Haiku順）
    pub fn get_all_sessions(&self) -> Vec<SessionData> {
        let mut sessions: Vec<SessionData> = self.sessions.values().cloned().collect();
        sessions.sort_by_key(|s| {
            let name = s.model_name.split_whitespace().next().unwrap_or("");
            match name {
                "Opus" => 0,
                "Sonnet" => 1,
                "Haiku" => 2,
                _ => 3,
            }
        });
        sessions
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
        assert_eq!(status.context_window.used_percentage, Some(62));
        assert_eq!(status.model.display_name, "Opus");
    }

    #[test]
    fn test_session_store_update() {
        let mut store = SessionStore::new();

        let status = StatusLineData {
            context_window: ContextWindow {
                used_percentage: Some(50),
                remaining_percentage: Some(50),
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
    fn test_usage_history_accumulation() {
        let mut store = SessionStore::new();

        // 5回更新して履歴が蓄積されることを確認
        for i in 0..5 {
            let status = StatusLineData {
                context_window: ContextWindow {
                    used_percentage: Some(i * 20),
                    remaining_percentage: Some(100 - i * 20),
                    context_window_size: 200000,
                    total_input_tokens: 100000,
                    total_output_tokens: 5000,
                },
                model: Model {
                    id: "claude-opus-4-6".to_string(),
                    display_name: "Opus".to_string(),
                },
                session_id: "session_hist".to_string(),
                cost: Cost {
                    total_cost_usd: 0.10,
                    total_duration_ms: 100000,
                },
            };
            store.update_from_status_line(status, "/home/user/project".to_string());
        }

        let sessions = store.get_all_sessions();
        let session = &sessions[0];
        assert_eq!(session.usage_history.len(), 5);
        assert_eq!(session.usage_history[0], 0.0);
        assert_eq!(session.usage_history[4], 80.0);
    }

    #[test]
    fn test_usage_history_max_samples_bound() {
        use crate::data::MAX_HISTORY_SAMPLES;
        let mut store = SessionStore::new();

        // MAX_HISTORY_SAMPLES + 5回更新
        for i in 0..(MAX_HISTORY_SAMPLES + 5) {
            let status = StatusLineData {
                context_window: ContextWindow {
                    used_percentage: Some(i as u32),
                    remaining_percentage: Some(100 - i as u32),
                    context_window_size: 200000,
                    total_input_tokens: 100000,
                    total_output_tokens: 5000,
                },
                model: Model {
                    id: "claude-opus-4-6".to_string(),
                    display_name: "Opus".to_string(),
                },
                session_id: "session_bound".to_string(),
                cost: Cost {
                    total_cost_usd: 0.10,
                    total_duration_ms: 100000,
                },
            };
            store.update_from_status_line(status, "/home/user/project".to_string());
        }

        let sessions = store.get_all_sessions();
        let session = &sessions[0];
        assert_eq!(session.usage_history.len(), MAX_HISTORY_SAMPLES);
        // 古いデータが押し出され、最後のMAX_HISTORY_SAMPLES個が残る
        assert_eq!(session.usage_history[0], 5.0); // 最初の5つが押し出された
        assert_eq!(
            *session.usage_history.back().unwrap(),
            (MAX_HISTORY_SAMPLES + 4) as f64
        );
    }

    #[test]
    fn test_usage_history_via_update_session() {
        use std::collections::VecDeque;
        let mut store = SessionStore::new();

        let mut history = VecDeque::new();
        history.push_back(10.0);
        history.push_back(20.0);

        let session = SessionData {
            session_id: "session_direct".to_string(),
            project_dir: "/tmp".to_string(),
            model_name: "Sonnet".to_string(),
            context_used_percent: 30,
            cost_usd: 0.05,
            duration_ms: 50000,
            peak_usage_percent: 30,
            subagent_completed_count: 0,
            last_updated_at: 0,
            usage_history: history,
        };

        store.update_session(session);

        let sessions = store.get_all_sessions();
        let s = &sessions[0];
        // update_sessionでは既存値にpush_backで30%が追加される
        assert_eq!(s.usage_history.len(), 3);
        assert_eq!(*s.usage_history.back().unwrap(), 30.0);
    }

    #[test]
    fn test_peak_usage_tracking() {
        let mut store = SessionStore::new();

        let status1 = StatusLineData {
            context_window: ContextWindow {
                used_percentage: Some(30),
                remaining_percentage: Some(30),
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
                used_percentage: Some(75),
                remaining_percentage: Some(25),
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
