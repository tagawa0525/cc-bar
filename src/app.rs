use crate::data::{SessionData, SessionStore};
use crate::message::Message;
use crate::watcher;
use cosmic::{
    app::{Command, Core},
    applet::Context,
    iced::{
        self,
        wayland::popup::{destroy_popup, get_popup},
        window, Alignment, Length, Subscription,
    },
    widget, Application, Element,
};

pub struct CcBar {
    core: Core,
    context: Context,
    popup: Option<window::Id>,
    session_store: SessionStore,
}

impl Application for CcBar {
    type Executor = cosmic::executor::Default;
    type Flags = ();
    type Message = Message;
    const APP_ID: &'static str = crate::config::APP_ID;

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, _flags: Self::Flags) -> (Self, Command<Self::Message>) {
        let context = Context::default();
        let app = CcBar {
            core,
            context,
            popup: None,
            session_store: SessionStore::new(),
        };
        (app, Command::none())
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::TogglePopup => {
                if let Some(id) = self.popup.take() {
                    destroy_popup(id)
                } else {
                    let new_id = window::Id::unique();
                    self.popup = Some(new_id);

                    let mut popup_settings = self.context.get_popup_settings(
                        window::Id::RESERVED,
                        new_id,
                        None,
                        None,
                        None,
                    );
                    popup_settings.positioner.size_limits = popup_settings
                        .positioner
                        .size_limits
                        .max_width(400.0)
                        .max_height(600.0);

                    get_popup(popup_settings)
                }
            }
            Message::SessionUpdate(session_id) => {
                // セッションファイルを読み込んで更新
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
                            .map(|p| p.file_name())
                            .and_then(|n| n.to_str())
                            .unwrap_or("Unknown")
                            .to_string();

                        self.session_store
                            .update_from_status_line(status, project_dir);
                    }
                }
                Command::none()
            }
            Message::Tick => {
                // 5分（300秒）以上更新がないセッションを除去
                self.session_store.remove_stale_sessions(300);
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<Self::Message> {
        let sessions = self.session_store.get_all_sessions();

        if sessions.is_empty() {
            // セッションなし：グレーアウトしたドーナツ
            let content = widget::container(
                widget::text("—")
                    .size(20)
                    .width(Length::Shrink)
                    .height(Length::Shrink)
                    .horizontal_alignment(Alignment::Center)
                    .vertical_alignment(Alignment::Center),
            )
            .width(Length::Fixed(48.0))
            .height(Length::Fixed(48.0))
            .center_x()
            .center_y();

            self.context
                .button_from_element(content, true)
                .on_press_down(Message::TogglePopup)
                .into()
        } else if sessions.len() == 1 {
            // 1セッション：ドーナツチャート表示
            let session = &sessions[0];
            let model_label = match session.model_name.as_str() {
                "Opus" => 'O',
                "Sonnet" => 'S',
                "Haiku" => 'H',
                _ => '?',
            };

            let chart = crate::chart::DonutChart::new(session.context_used_percent, model_label);
            let content = chart.view();

            self.context
                .button_from_element(content, true)
                .on_press_down(Message::TogglePopup)
                .into()
        } else {
            // 複数セッション：複数のドーナツを横並び
            let mut row = widget::row().spacing(4);

            for session in sessions.iter().take(3) {
                let model_label = match session.model_name.as_str() {
                    "Opus" => 'O',
                    "Sonnet" => 'S',
                    "Haiku" => 'H',
                    _ => '?',
                };

                let chart =
                    crate::chart::DonutChart::new(session.context_used_percent, model_label);
                row = row.push(chart.view());
            }

            if sessions.len() > 3 {
                row = row.push(widget::text(format!("+{}", sessions.len() - 3)).size(12));
            }

            self.context
                .button_from_element(row, true)
                .on_press_down(Message::TogglePopup)
                .into()
        }
    }

    fn view_window(&self, id: window::Id) -> Element<Self::Message> {
        if self.popup == Some(id) {
            let mut content = widget::column()
                .padding(16)
                .spacing(12)
                .push(widget::text("Claude Code Sessions").size(20));

            let sessions = self.session_store.get_all_sessions();

            if sessions.is_empty() {
                content = content.push(widget::text("セッション監視中...").size(14));
            } else {
                for session in sessions {
                    let model_label = match session.model_name.as_str() {
                        "Opus" => "O",
                        "Sonnet" => "S",
                        "Haiku" => "H",
                        _ => "?",
                    };

                    let session_info = widget::column()
                        .spacing(4)
                        .push(
                            widget::text(format!(
                                "{} [{}] - {}%",
                                model_label, session.project_dir, session.context_used_percent
                            ))
                            .size(14),
                        )
                        .push(
                            widget::text(format!(
                                "Cost: ${:.3} | Duration: {:.1}s | Peak: {}%",
                                session.cost_usd,
                                session.duration_ms as f64 / 1000.0,
                                session.peak_usage_percent
                            ))
                            .size(12),
                        );

                    content = content.push(session_info);
                }
            }

            self.context.popup_container(content).into()
        } else {
            widget::text("").into()
        }
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        // ファイル監視とタイマーを統合
        Subscription::batch(vec![watcher::watch_sessions(), watcher::tick_timer()])
    }
}
