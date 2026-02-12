use crate::data::SessionStore;
use crate::message::Message;
use crate::watcher;
use cosmic::{
    app::{Core, Task},
    iced::{
        platform_specific::shell::wayland::commands::popup::{destroy_popup, get_popup},
        window, Subscription,
    },
    widget, Application, Element,
};

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
            Message::SessionUpdate(session_id) => {
                let runtime_dir = dirs::runtime_dir().unwrap_or_else(|| {
                    let uid = unsafe { libc::getuid() };
                    std::path::PathBuf::from(format!("/run/user/{}", uid))
                });
                let session_file = runtime_dir
                    .join("cc-bar")
                    .join("sessions")
                    .join(format!("{}.json", session_id));

                if let Ok(content) = std::fs::read_to_string(&session_file) {
                    if let Ok(status) =
                        serde_json::from_str::<crate::data::StatusLineData>(&content)
                    {
                        let project_dir = session_file
                            .parent()
                            .and_then(|p| p.parent())
                            .and_then(|p| p.file_name())
                            .and_then(|n| n.to_str())
                            .unwrap_or("Unknown")
                            .to_string();

                        self.session_store
                            .update_from_status_line(status, project_dir);
                    }
                }
                Task::none()
            }
            Message::Tick => {
                self.session_store.remove_stale_sessions(300);
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
            let chart_size = suggested.0 as f32;
            let mut row = widget::row().spacing(4);

            for session in sessions.iter().take(3) {
                let model_label = match session.model_name.as_str() {
                    "Opus" => 'O',
                    "Sonnet" => 'S',
                    "Haiku" => 'H',
                    _ => '?',
                };

                row = row.push(crate::chart::donut_view::<Message>(
                    session.context_used_percent,
                    model_label,
                    chart_size,
                ));
            }

            if sessions.len() > 3 {
                row = row.push(widget::text(format!("+{}", sessions.len() - 3)).size(12));
            }

            self.core
                .applet
                .button_from_element(row, true)
                .on_press_down(Message::TogglePopup)
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
                let model_label = match session.model_name.as_str() {
                    "Opus" => "Opus",
                    "Sonnet" => "Sonnet",
                    "Haiku" => "Haiku",
                    other => other,
                };

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
