use cosmic::iced::{Color, Length};
use cosmic::widget::container;
use cosmic::{Element, Theme};

/// ドーナツチャート（container による円形インジケータ）
/// 使用率を背景色で表現し、モデル型で濃淡を調整
pub fn donut_view<'a, M: Clone + 'a>(
    percentage: u32,
    model_type: &str,
    size: f32,
) -> Element<'a, M> {
    let percentage = percentage.min(100);

    // 使用率に応じた基本色
    let (r, g, b) = match percentage {
        0..=69 => (0.2, 0.6, 0.2),  // 緑
        70..=89 => (0.7, 0.5, 0.0), // 黄
        _ => (0.7, 0.2, 0.2),       // 赤
    };

    // モデル型による濃淡係数
    let intensity = match model_type {
        "Opus" => 1.0,    // 最も濃い
        "Sonnet" => 0.65, // 中間
        "Haiku" => 0.35,  // 最も薄い
        _ => 0.65,
    };

    let bg_color = Color::from_rgb(r * intensity, g * intensity, b * intensity);

    // 空のコンテナ（テキスト非表示）
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
                    radius: [size / 2.0; 4].into(),
                    width: 2.0,
                    color: cosmic.background.divider.into(),
                },
                ..Default::default()
            }
        }))
        .into()
}
