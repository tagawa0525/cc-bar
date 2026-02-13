use cosmic::iced::{Color, Length};
use cosmic::widget::container;
use cosmic::{Element, Theme};

/// R付き四角形インジケータ
/// モデル型で基本色、使用率で濃さを調整
pub fn donut_view<'a, M: Clone + 'a>(
    percentage: u32,
    model_type: &str,
    size: f32,
) -> Element<'a, M> {
    let percentage = percentage.min(100);

    // モデル型ごとの基本色（RGB）
    let (r, g, b) = match model_type {
        "Opus" => (1.0, 0.55, 0.1),  // オレンジ
        "Sonnet" => (0.2, 0.4, 0.9), // 青
        "Haiku" => (0.1, 0.7, 0.3),  // 緑
        _ => (0.5, 0.5, 0.5),        // その他: グレー
    };

    // 使用率による濃さ調整（低→暗い、高→明るい）
    let intensity = 0.4 + (percentage as f32 / 100.0) * 0.6;
    let bg_color = Color::from_rgb(r * intensity, g * intensity, b * intensity);

    container(cosmic::widget::text(""))
        .width(Length::Fixed(size))
        .height(Length::Fixed(size))
        .center_x(Length::Fixed(size))
        .center_y(Length::Fixed(size))
        .class(cosmic::style::Container::custom(move |theme: &Theme| {
            let cosmic = theme.cosmic();
            container::Style {
                background: Some(bg_color.into()),
                border: cosmic::iced::Border {
                    radius: [4.0; 4].into(),
                    width: 0.0,
                    color: cosmic.background.divider.into(),
                },
                ..Default::default()
            }
        }))
        .into()
}
