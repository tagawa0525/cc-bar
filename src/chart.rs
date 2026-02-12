use cosmic::iced::{Color, Length};
use cosmic::widget::{container, text};
use cosmic::{Element, Theme};

/// ドーナツチャート（container + text による円形インジケータ）
/// 使用率を背景色で表現し、中央にモデル名の頭文字を表示
pub fn donut_view<'a, M: Clone + 'a>(
    percentage: u32,
    model_label: char,
    size: f32,
) -> Element<'a, M> {
    let percentage = percentage.min(100);

    // 使用率に応じた色
    let bg_color = match percentage {
        0..=69 => Color::from_rgb(0.2, 0.6, 0.2),  // 緑
        70..=89 => Color::from_rgb(0.7, 0.5, 0.0), // 黄
        _ => Color::from_rgb(0.7, 0.2, 0.2),       // 赤
    };

    let label: Element<'a, M> = text::body(model_label.to_string()).into();

    container(label)
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
