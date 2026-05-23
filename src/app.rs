use crate::data::SessionStore;
use crate::message::Message;
use crate::watcher;
use cosmic::{
    app::{Core, Task},
    iced::{
        platform_specific::shell::wayland::commands::popup::{destroy_popup, get_popup},
        window, Alignment, Subscription,
    },
    widget::{self, Row},
    Application, Element,
};

/// display_name（例: "Opus 4.6", "Sonnet"）からモデル型を返す
fn model_type(display_name: &str) -> &'static str {
    let name = display_name.split_whitespace().next().unwrap_or("");
    match name {
        "Opus" => "Opus",
        "Sonnet" => "Sonnet",
        "Haiku" => "Haiku",
        _ => "Unknown",
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
                        if content.is_empty() {
                            crate::log!("cc-bar: file empty for {}", session_id);
                        } else {
                            match serde_json::from_str::<crate::data::StatusLineData>(&content) {
                                Ok(status) => {
                                    // NOTE: The session file path is always `<runtime_dir>/cc-bar/sessions/<id>.json`,
                                    // so deriving a project directory name from its parent directories would always
                                    // yield "cc-bar" and not the actual project. Until project information is
                                    // provided via a reliable source (e.g., included in the JSON by the relay),
                                    // we explicitly mark the project as unknown here.
                                    let project_dir = "Unknown".to_string();
                                    crate::log!(
                                        "cc-bar: parsed {}, usage={:?}%, total={}",
                                        session_id,
                                        status.context_window.used_percentage,
                                        self.session_store.session_count() + 1
                                    );
                                    self.session_store
                                        .update_from_status_line(status, project_dir);
                                }
                                Err(e) => {
                                    crate::log!(
                                        "cc-bar: JSON parse error for {}: {}",
                                        session_id,
                                        e
                                    );
                                }
                            }
                        }
                    }
                    Err(e) => {
                        crate::log!("cc-bar: file read error for {}: {}", session_id, e);
                    }
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
                self.session_store
                    .remove_stale_by_mtime(&sessions_dir, crate::data::STALE_THRESHOLD_SECS);
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
            let horizontal = self.core.applet.is_horizontal();
            let (w, h) = self.core.applet.suggested_size(false);
            // minimonと同様にwidth/heightを別々に使用
            let (chart_w, chart_h) = if horizontal {
                (h as f32, h as f32)
            } else {
                (w as f32, w as f32)
            };
            let charts: Vec<Element<Message>> = sessions
                .iter()
                .map(|session| {
                    let model_type_str = model_type(&session.model_name);
                    crate::chart::line_chart_view::<Message>(
                        &session.usage_history,
                        model_type_str,
                        chart_w,
                        chart_h,
                    )
                })
                .collect();

            let wrapper: Element<Message> = if horizontal {
                Row::with_children(charts)
                    .align_y(Alignment::Center)
                    .spacing(2.0)
                    .into()
            } else {
                widget::column::with_children(charts)
                    .align_x(Alignment::Center)
                    .spacing(2.0)
                    .into()
            };

            let padding = self.core.applet.suggested_padding(true);
            let button = widget::button::custom(wrapper)
                .padding(if horizontal {
                    [0, padding.1]
                } else {
                    [padding.0, 0]
                })
                .class(cosmic::theme::Button::AppletIcon)
                .on_press(Message::TogglePopup);

            self.core
                .applet
                .autosize_window(widget::container(button))
                .into()
        }
    }

    fn view_window(&self, id: window::Id) -> Element<'_, Self::Message> {
        if self.popup != Some(id) {
            return widget::text("").into();
        }

        let sessions = self.session_store.get_all_sessions();

        let mut content = widget::Column::new()
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

                let session_row = widget::Column::new()
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

    fn style(&self) -> Option<cosmic::iced::theme::Style> {
        Some(cosmic::applet::style())
    }
}
