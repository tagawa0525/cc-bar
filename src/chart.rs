use std::collections::VecDeque;
use std::fmt::Write;

use cosmic::iced::Length;
use cosmic::Element;

struct ChartColors {
    line: &'static str,
    fill: &'static str,
    background: &'static str,
    frame: &'static str,
}

fn colors_for_model(model_type: &str) -> ChartColors {
    match model_type {
        "Opus" => ChartColors {
            line: "#FF8C1A",
            fill: "#FF8C1A60",
            background: "#1A1A1AE0",
            frame: "#FFFFFF",
        },
        "Sonnet" => ChartColors {
            line: "#3366E6",
            fill: "#3366E660",
            background: "#1A1A1AE0",
            frame: "#FFFFFF",
        },
        "Haiku" => ChartColors {
            line: "#1AB34D",
            fill: "#1AB34D60",
            background: "#1A1A1AE0",
            frame: "#FFFFFF",
        },
        _ => ChartColors {
            line: "#808080",
            fill: "#80808060",
            background: "#1A1A1AE0",
            frame: "#FFFFFF",
        },
    }
}

fn generate_line_svg(samples: &VecDeque<f64>, model_type: &str) -> String {
    let colors = colors_for_model(model_type);

    let mut svg = String::with_capacity(1024);
    let _ = write!(
        svg,
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 42 42">"#
    );

    // Clip path with rounded corners
    let _ = write!(
        svg,
        r#"<defs><clipPath id="rc"><rect width="42" height="42" rx="7"/></clipPath></defs>"#
    );
    let _ = write!(svg, r#"<g clip-path="url(#rc)">"#);

    // Background
    let _ = write!(
        svg,
        r#"<rect width="42" height="42" fill="{}"/>"#,
        colors.background
    );

    // Frame border
    let _ = write!(
        svg,
        r#"<rect x="0.5" y="0.5" width="41" height="41" rx="6.5" fill="none" stroke="{}" stroke-width="1"/>"#,
        colors.frame
    );

    if samples.is_empty() {
        let _ = write!(svg, "</g></svg>");
        return svg;
    }

    // Build polyline points and polygon points
    let mut line_points = String::new();
    let mut poly_points = String::new();

    for (i, &value) in samples.iter().enumerate() {
        let value = value.clamp(0.0, 100.0);
        let x = (i * 2) + 1;
        let y = 41.0 - (40.0 / 100.0 * value);
        if i > 0 {
            line_points.push(' ');
            poly_points.push(' ');
        }
        let _ = write!(line_points, "{},{:.1}", x, y);
        let _ = write!(poly_points, "{},{:.1}", x, y);
    }

    // Close polygon to bottom edge for fill
    let last_x = (samples.len() - 1) * 2 + 1;
    let first_x = 1;
    let _ = write!(poly_points, " {},42 {},42", last_x, first_x);

    // Fill area
    let _ = write!(
        svg,
        r#"<polygon points="{}" fill="{}" stroke="none"/>"#,
        poly_points, colors.fill
    );

    // Line stroke
    let _ = write!(
        svg,
        r#"<polyline points="{}" fill="none" stroke="{}" stroke-width="1.5" stroke-linejoin="round" stroke-linecap="round"/>"#,
        line_points, colors.line
    );

    let _ = write!(svg, "</g></svg>");
    svg
}

pub fn line_chart_view<'a, M: Clone + 'a>(
    samples: &VecDeque<f64>,
    model_type: &str,
    width: f32,
    height: f32,
) -> Element<'a, M> {
    let svg = generate_line_svg(samples, model_type);
    cosmic::widget::icon::from_svg_bytes(svg.into_bytes())
        .icon()
        .width(Length::Fixed(width))
        .height(Length::Fixed(height))
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_line_svg_empty() {
        let samples = VecDeque::new();
        let svg = generate_line_svg(&samples, "Opus");
        assert!(svg.contains("viewBox=\"0 0 42 42\""));
        assert!(!svg.contains("polyline"));
    }

    #[test]
    fn test_generate_line_svg_single_sample() {
        let mut samples = VecDeque::new();
        samples.push_back(50.0);
        let svg = generate_line_svg(&samples, "Sonnet");
        assert!(svg.contains("polyline"));
        assert!(svg.contains("#3366E6"));
    }

    #[test]
    fn test_generate_line_svg_full_samples() {
        let mut samples = VecDeque::new();
        for i in 0..21 {
            samples.push_back(i as f64 * 5.0);
        }
        let svg = generate_line_svg(&samples, "Haiku");
        assert!(svg.contains("#1AB34D"));
        assert!(svg.contains("polygon"));
        assert!(svg.contains("polyline"));
        // Last point x = (20 * 2) + 1 = 41
        assert!(svg.contains("41,"));
    }

    #[test]
    fn test_colors_for_model() {
        let opus = colors_for_model("Opus");
        assert_eq!(opus.line, "#FF8C1A");

        let sonnet = colors_for_model("Sonnet");
        assert_eq!(sonnet.line, "#3366E6");

        let haiku = colors_for_model("Haiku");
        assert_eq!(haiku.line, "#1AB34D");

        let unknown = colors_for_model("Unknown");
        assert_eq!(unknown.line, "#808080");
    }
}
