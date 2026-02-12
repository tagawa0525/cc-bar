use cosmic::iced::{Color, Length, Point, Radians, Rectangle};
use cosmic::iced_widget::canvas;
use cosmic::Element;
use std::f32::consts::PI;

/// 円グラフ（Canvas によるパイチャート）
/// 使用率を扇形で表現し、モデル型で背景色を区別
pub fn donut_view<'a, M: 'a>(percentage: u32, model_type: &str, size: f32) -> Element<'a, M> {
    let percentage = percentage.min(100);

    // モデル型ごとの背景色
    let bg_color = match model_type {
        "Opus" => Some(Color::from_rgb(0.9, 0.55, 0.1)), // オレンジ
        "Sonnet" => Some(Color::from_rgb(0.2, 0.4, 0.8)), // 青
        _ => None,                                       // Haiku等: なし
    };

    // 使用率に応じたグラフ色（緑→黄→赤）
    let fg_color = match percentage {
        0..=69 => Color::from_rgb(0.2, 0.7, 0.2),
        70..=89 => Color::from_rgb(0.8, 0.7, 0.0),
        _ => Color::from_rgb(0.8, 0.2, 0.2),
    };

    canvas::Canvas::<_, M, cosmic::Theme, cosmic::Renderer>::new(PieChart {
        percentage,
        bg_color,
        fg_color,
    })
    .width(Length::Fixed(size))
    .height(Length::Fixed(size))
    .into()
}

struct PieChart {
    percentage: u32,
    bg_color: Option<Color>,
    fg_color: Color,
}

impl<M> canvas::Program<M, cosmic::Theme, cosmic::Renderer> for PieChart {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &cosmic::Renderer,
        _theme: &cosmic::Theme,
        bounds: Rectangle,
        _cursor: cosmic::iced::mouse::Cursor,
    ) -> Vec<canvas::Geometry<cosmic::Renderer>> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let center = Point::new(bounds.width / 2.0, bounds.height / 2.0);
        let radius = bounds.width.min(bounds.height) / 2.0 - 0.5;

        // 背景円（モデル型の色）
        if let Some(bg) = self.bg_color {
            frame.fill(&canvas::Path::circle(center, radius), bg);
        }

        // 使用率の扇形
        if self.percentage > 0 {
            let angle = (self.percentage as f32 / 100.0) * 2.0 * PI;
            let start = -PI / 2.0; // 12時方向から開始

            let pie = canvas::Path::new(|b| {
                b.move_to(center);
                b.arc(canvas::path::Arc {
                    center,
                    radius,
                    start_angle: Radians(start),
                    end_angle: Radians(start + angle),
                });
                b.line_to(center);
            });

            frame.fill(&pie, self.fg_color);
        }

        vec![frame.into_geometry()]
    }
}
