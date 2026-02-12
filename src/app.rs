use crate::message::Message;
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
            Message::SessionUpdate(_session_id) => {
                // TODO: セッション更新処理
                Command::none()
            }
            Message::Tick => {
                // TODO: staleセッション除去
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<Self::Message> {
        let content = widget::text("CC")
            .size(16)
            .width(Length::Shrink)
            .height(Length::Shrink)
            .horizontal_alignment(Alignment::Center)
            .vertical_alignment(Alignment::Center);

        self.context
            .button_from_element(content, true)
            .on_press_down(Message::TogglePopup)
            .into()
    }

    fn view_window(&self, id: window::Id) -> Element<Self::Message> {
        if self.popup == Some(id) {
            let content = widget::column()
                .padding(16)
                .spacing(8)
                .push(widget::text("Claude Code Sessions").size(20))
                .push(widget::text("セッション監視中..."));

            self.context.popup_container(content).into()
        } else {
            widget::text("").into()
        }
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        // TODO: ファイル監視とタイマーのSubscription
        Subscription::none()
    }
}
