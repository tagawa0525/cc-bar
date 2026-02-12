use cosmic::iced::mouse;
use cosmic::iced::widget::canvas::{self, Cache, Canvas, Path, Stroke};
use cosmic::iced::{Color, Element, Length, Point, Rectangle, Size};
use std::f32::consts::PI;

/// ドーナツチャートウィジェット（canvas描画）
/// コンテキスト使用率をドーナツチャートで表示し、中央にモデル名の頭文字を描画
#[derive(Debug)]
pub struct DonutChart {
    percentage: u32,
    model_label: char,
    cache: Cache<cosmic::Theme>,
}

impl DonutChart {
    pub fn new(percentage: u32, model_label: char) -> Self {
        DonutChart {
            percentage: percentage.min(100),
            model_label,
            cache: Cache::new(),
        }
    }

    /// 使用率に応じた色を返す
    fn arc_color(&self) -> Color {
        match self.percentage {
            0..=69 => Color::from_rgb(0.3, 0.8, 0.3),  // 緑
            70..=89 => Color::from_rgb(1.0, 0.8, 0.0), // 黄
            _ => Color::from_rgb(1.0, 0.3, 0.3),       // 赤
        }
    }

    pub fn view(&self) -> Element<'_, ()> {
        Canvas::new(self)
            .width(Length::Fixed(32.0))
            .height(Length::Fixed(32.0))
            .into()
    }
}

impl canvas::Program<(), cosmic::Theme> for DonutChart {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &cosmic::Renderer,
        _theme: &cosmic::Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry<cosmic::Renderer>> {
        let geometry = self.cache.draw(renderer, bounds.size(), |frame| {
            let center = frame.center();
            let radius = bounds.width.min(bounds.height) / 2.0 - 2.0;
            let stroke_width = radius * 0.3;
            let mid_radius = radius - stroke_width / 2.0;

            // 背景円弧（グレー）
            let bg_circle = Path::circle(center, mid_radius);
            frame.stroke(
                &bg_circle,
                Stroke {
                    width: stroke_width,
                    style: canvas::stroke::Style::Solid(Color::from_rgba(0.4, 0.4, 0.4, 0.5)),
                    line_cap: canvas::LineCap::Round,
                    ..Stroke::default()
                },
            );

            // 使用率の円弧
            if self.percentage > 0 {
                let angle = (self.percentage as f32 / 100.0) * 2.0 * PI;
                let arc = Path::new(|builder| {
                    let start_angle = -PI / 2.0;
                    let end_angle = start_angle + angle;

                    // 弧を手動で近似（多角形ライン）
                    let segments = 64;
                    let step = angle / segments as f32;

                    let start_x = center.x + mid_radius * start_angle.cos();
                    let start_y = center.y + mid_radius * start_angle.sin();
                    builder.move_to(Point::new(start_x, start_y));

                    for i in 1..=segments {
                        let a = start_angle + step * i as f32;
                        if a > end_angle {
                            break;
                        }
                        let x = center.x + mid_radius * a.cos();
                        let y = center.y + mid_radius * a.sin();
                        builder.line_to(Point::new(x, y));
                    }

                    // 最終点
                    let x = center.x + mid_radius * end_angle.cos();
                    let y = center.y + mid_radius * end_angle.sin();
                    builder.line_to(Point::new(x, y));
                });

                frame.stroke(
                    &arc,
                    Stroke {
                        width: stroke_width,
                        style: canvas::stroke::Style::Solid(self.arc_color()),
                        line_cap: canvas::LineCap::Round,
                        ..Stroke::default()
                    },
                );
            }

            // 中央にモデル名ラベル
            frame.fill_text(canvas::Text {
                content: self.model_label.to_string(),
                position: center,
                color: Color::from_rgb(0.9, 0.9, 0.9),
                size: cosmic::iced::Pixels(12.0),
                font: cosmic::iced::Font::default(),
                horizontal_alignment: cosmic::iced::alignment::Horizontal::Center,
                vertical_alignment: cosmic::iced::alignment::Vertical::Center,
                ..canvas::Text::default()
            });
        });

        vec![geometry]
    }
}
