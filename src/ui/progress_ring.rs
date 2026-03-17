use egui::{Color32, Pos2, Stroke};
use std::f32::consts::{PI, TAU};

/// Draw a circular progress ring.
/// `progress` is 0.0..=1.0, `center` is screen-space, `radius` is in logical pixels.
pub fn render_progress_ring(
    painter: &egui::Painter,
    center: Pos2,
    radius: f32,
    progress: f32,
    fg_color: Color32,
    bg_color: Color32,
    stroke_width: f32,
) {
    // Background track (full circle)
    let bg_points = arc_points(center, radius, 0.0, TAU, 64);
    painter.add(egui::Shape::line(bg_points, Stroke::new(stroke_width, bg_color)));

    // Foreground arc (partial, starting from top = -PI/2)
    if progress > 0.001 {
        let sweep = TAU * progress.clamp(0.0, 1.0);
        let segments = ((64.0 * progress).max(4.0)) as usize;
        let fg_points = arc_points(center, radius, -PI / 2.0, sweep, segments);
        painter.add(egui::Shape::line(
            fg_points,
            Stroke::new(stroke_width, fg_color),
        ));
    }
}

/// Generate points along a circular arc.
fn arc_points(center: Pos2, radius: f32, start_angle: f32, sweep: f32, segments: usize) -> Vec<Pos2> {
    (0..=segments)
        .map(|i| {
            let t = i as f32 / segments as f32;
            let angle = start_angle + sweep * t;
            Pos2::new(
                center.x + radius * angle.cos(),
                center.y + radius * angle.sin(),
            )
        })
        .collect()
}
