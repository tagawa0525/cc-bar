use crate::data::SessionStore;
use crate::message::Message;
use crate::watcher;
use cosmic::{
    app::{Core, Task},
    iced::{
        platform_specific::shell::wayland::commands::popup::{destroy_popup, get_popup},
        window, Limits, Subscription,
    },
    widget::{self, Id},
    Application, Element,
};
use std::sync::LazyLock;

static AUTOSIZE_MAIN_ID: LazyLock<Id> = LazyLock::new(|| Id::new("autosize-main"));

/// display_name（例: "Opus 4.6", "Sonnet"）からモデル型を返す
fn model_type(display_name: &str) -> &'static str {
    let name = display_name.split_whitespace().next().unwrap_or("");
    match name {
        "Opus" => "Opus",
        "Sonnet" => "Sonnet",
        "Haiku" => "Haiku",
        _ => "Sonnet",
    }
}

pub struct CcBar {
    core: Core,
    popup: Option<window::Id>,
    session_store: SessionStore,
}

impl Application for CcBar {
    type Executor = cosmic::SingleThreadExecutor;
    type Flags = ();
    type Message = Message;
    const APP_ID: &'static str = crate::config::APP_ID;

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, _flags: Self::Flags) -> (Self, Task<Self::Message>) {
        crate::log!("cc-bar: init called");
        let app = CcBar {
            core,
            popup: None,
            session_store: SessionStore::new(),
        };
        (app, Task::none())
    }

    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        match message {
            Message::TogglePopup => {
                if let Some(id) = self.popup.take() {
                    destroy_popup(id)
                } else {
                    let new_id = window::Id::unique();
                    self.popup = Some(new_id);

                    let popup_settings = self.core.applet.get_popup_settings(
                        self.core.main_window_id().unwrap(),
                        new_id,
                        None,
                        None,
                        None,
                    );

                    get_popup(popup_settings)
                }
            }
            Message::SessionUpdate(ref session_id) => {
                crate::log!("cc-bar: SessionUpdate received for {}", session_id);
                let runtime_dir = dirs::runtime_dir().unwrap_or_else(|| {
                    let uid = unsafe { libc::getuid() };
                    std::path::PathBuf::from(format!("/run/user/{}", uid))
                });
                let session_file = runtime_dir
                    .join("cc-bar")
                    .join("sessions")
                    .join(format!("{}.json", session_id));

                match std::fs::read_to_string(&session_file) {
                    Ok(content) => {
                        match serde_json::from_str::<crate::data::StatusLineData>(&content) {
                            Ok(status) => {
                                let project_dir = session_file
                                    .parent()
                                    .and_then(|p| p.parent())
                                    .and_then(|p| p.file_name())
                                    .and_then(|n| n.to_str())
                                    .unwrap_or("Unknown")
                                    .to_string();
                                crate::log!(
                                    "cc-bar: parsed session, usage={:?}%",
                                    status.context_window.used_percentage
                                );
                                self.session_store
                                    .update_from_status_line(status, project_dir);
                            }
                            Err(e) => crate::log!("cc-bar: JSON parse error: {}", e),
                        }
                    }
                    Err(e) => crate::log!("cc-bar: file read error: {}", e),
                }
                Task::none()
            }
            Message::SessionRemoved(ref session_id) => {
                self.session_store.remove_session(session_id);
                Task::none()
            }
            Message::Tick => {
                let runtime_dir = dirs::runtime_dir().unwrap_or_else(|| {
                    let uid = unsafe { libc::getuid() };
                    std::path::PathBuf::from(format!("/run/user/{}", uid))
                });
                let sessions_dir = runtime_dir.join("cc-bar").join("sessions");
                self.session_store.remove_stale_by_mtime(&sessions_dir, 15);
                Task::none()
            }
            Message::CloseRequested(id) => {
                if Some(id) == self.popup {
                    self.popup = None;
                }
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let sessions = self.session_store.get_all_sessions();

        if sessions.is_empty() {
            self.core
                .applet
                .icon_button("dialog-information-symbolic")
                .on_press_down(Message::TogglePopup)
                .into()
        } else {
            let suggested = self.core.applet.suggested_size(true);
            let chart_size = suggested.1 as f32;
            let mut row = widget::row().spacing(2);

            for session in &sessions {
                let model_type_str = model_type(&session.model_name);

                row = row.push(crate::chart::donut_view::<Message>(
                    session.context_used_percent,
                    model_type_str,
                    chart_size,
                ));
            }

            let content = widget::mouse_area(row).on_press(Message::TogglePopup);

            let mut limits = Limits::NONE.min_width(1.).min_height(1.);
            if let Some(b) = self.core.applet.suggested_bounds {
                if b.width as i32 > 0 {
                    limits = limits.max_width(b.width);
                }
                if b.height as i32 > 0 {
                    limits = limits.max_height(b.height);
                }
            }

            widget::autosize::autosize(
                widget::container(content).padding(0),
                AUTOSIZE_MAIN_ID.clone(),
            )
            .limits(limits)
            .into()
        }
    }

    fn view_window(&self, id: window::Id) -> Element<'_, Self::Message> {
        if self.popup != Some(id) {
            return widget::text("").into();
        }

        let sessions = self.session_store.get_all_sessions();

        let mut content = widget::column()
            .padding([8, 0])
            .spacing(4)
            .push(cosmic::applet::padded_control(widget::text::body(
                "Claude Code Sessions",
            )))
            .push(cosmic::applet::padded_control(
                widget::divider::horizontal::default(),
            ));

        if sessions.is_empty() {
            content = content.push(cosmic::applet::padded_control(widget::text::caption(
                "No active sessions",
            )));
        } else {
            for session in sessions {
                let model_label = &session.model_name;

                let duration_secs = session.duration_ms / 1000;
                let duration_display = if duration_secs >= 3600 {
                    format!("{}h{}m", duration_secs / 3600, (duration_secs % 3600) / 60)
                } else if duration_secs >= 60 {
                    format!("{}m{}s", duration_secs / 60, duration_secs % 60)
                } else {
                    format!("{}s", duration_secs)
                };

                let session_row = widget::column()
                    .spacing(2)
                    .push(widget::text::body(format!(
                        "{} | {} | {}%",
                        model_label, session.project_dir, session.context_used_percent
                    )))
                    .push(widget::text::caption(format!(
                        "${:.3} | {} | Peak {}% | Agents: {}",
                        session.cost_usd,
                        duration_display,
                        session.peak_usage_percent,
                        session.subagent_completed_count,
                    )));

                content = content.push(cosmic::applet::padded_control(session_row));
            }
        }

        self.core.applet.popup_container(content).into()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        Subscription::batch(vec![watcher::watch_sessions(), watcher::tick_timer()])
    }

    fn on_close_requested(&self, id: window::Id) -> Option<Self::Message> {
        Some(Message::CloseRequested(id))
    }

    fn style(&self) -> Option<cosmic::iced_runtime::Appearance> {
        Some(cosmic::applet::style())
    }
}
