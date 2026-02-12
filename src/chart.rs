use cosmic::iced::Alignment;
use cosmic::iced::{Color, Element, Length};
use cosmic::widget::{container, text};

/// ドーナツチャート表現（簡易版）
/// 実際のcanvas描画の代わりに、テキスト+背景色で表現
#[derive(Debug, Clone, Copy)]
pub struct DonutChart {
    pub percentage: u32,
    pub model_label: char,
}

impl DonutChart {
    pub fn new(percentage: u32, model_label: char) -> Self {
        DonutChart {
            percentage: percentage.min(100),
            model_label,
        }
    }

    pub fn view(&self) -> Element<'static, ()> {
        // 色の選択（使用率に基づく）
        let (bg_color, text_color) = match self.percentage {
            0..=69 => (
                Color::from_rgb(0.2, 0.5, 0.2),
                Color::from_rgb(0.9, 0.9, 0.9),
            ), // 緑
            70..=89 => (
                Color::from_rgb(0.6, 0.4, 0.1),
                Color::from_rgb(0.9, 0.9, 0.9),
            ), // 黄
            _ => (
                Color::from_rgb(0.5, 0.2, 0.2),
                Color::from_rgb(0.9, 0.9, 0.9),
            ), // 赤
        };

        let label = text(self.model_label)
            .size(14)
            .width(Length::Fixed(44.0))
            .height(Length::Fixed(44.0))
            .horizontal_alignment(Alignment::Center)
            .vertical_alignment(Alignment::Center)
            .color(text_color);

        container(label)
            .width(Length::Fixed(44.0))
            .height(Length::Fixed(44.0))
            .center_x()
            .center_y()
            .style(move |_theme| cosmic::widget::container::Style {
                background: Some(bg_color.into()),
                border: cosmic::iced::Border {
                    radius: [22.0; 4].into(),
                    width: 1.0,
                    color: Color::from_rgb(0.4, 0.4, 0.4),
                },
                ..Default::default()
            })
            .into()
    }
}
