use std::time::Instant;

use bevy::prelude::*;

/// A bevy resource to track widget metrics.
#[derive(Resource, Default)]
pub struct WidgetMetrics {
    total_widgets_rendered: usize,
    total_widgets_rendered_last_frame: usize,
    rendered_avg_buffer: Vec<usize>,

    quads_displayed: usize,
    quads_displayed_since_last_frame: usize,
    quads_avg_buffer: Vec<usize>,
}

pub struct LastUpdated(Instant);
impl Default for LastUpdated {
    fn default() -> Self {
        Self(Instant::now())
    }
}

impl WidgetMetrics {
    pub fn get_widgets_rendered(&self) -> usize {
        self.total_widgets_rendered
    }

    pub fn get_widgets_rendered_since_last_frame(&self) -> usize {
        self.total_widgets_rendered_last_frame
    }

    pub fn get_average_widgets_rendered_per_frame(&self) -> f32 {
        self.rendered_avg_buffer
            .iter()
            .map(|v| *v as f32)
            .sum::<f32>()
            / self.rendered_avg_buffer.len() as f32
    }

    pub(crate) fn increase_counts(&mut self) {
        self.total_widgets_rendered += 1;
        self.total_widgets_rendered_last_frame += 1;
    }

    pub(crate) fn clear_last_frame(&mut self) {
        self.total_widgets_rendered_last_frame = 0;
    }

    pub(crate) fn commit_frame(&mut self) {
        if self.rendered_avg_buffer.len() > 100 {
            self.rendered_avg_buffer.remove(0);
        }
        self.rendered_avg_buffer
            .push(self.total_widgets_rendered_last_frame);
    }

    pub fn get_quads_displayed(&self) -> usize {
        self.quads_displayed
    }

    pub fn get_quads_displayed_since_last_frame(&self) -> usize {
        self.quads_displayed_since_last_frame
    }

    pub fn get_average_quads_displayed_per_frame(&self) -> f32 {
        self.quads_avg_buffer.iter().map(|v| *v as f32).sum::<f32>()
            / self.quads_avg_buffer.len() as f32
    }

    pub(crate) fn increase_quad_counts(&mut self) {
        self.quads_displayed += 1;
        self.quads_displayed_since_last_frame += 1;
    }

    pub(crate) fn clear_quad_last_frame(&mut self) {
        self.quads_displayed_since_last_frame = 0;
    }

    pub(crate) fn commit_quad_frame(&mut self) {
        if self.quads_avg_buffer.len() > 100 {
            self.quads_avg_buffer.remove(0);
        }
        self.quads_avg_buffer
            .push(self.quads_displayed_since_last_frame);
    }

    /// A system that prints widget metrics!
    pub fn print_metrics_x_seconds(
        metrics: Res<WidgetMetrics>,
        mut last_update: Local<LastUpdated>,
    ) {
        if last_update.0.elapsed().as_secs_f32() > 5.0 {
            last_update.0 = Instant::now();
            info!(
                r#"
========================Woodpecker UI Metrics========================
Total Widgets Rendered: {},
Widgets Rendered Last Frame: {},
Average Rendered over 100 frames: {},
Total Quads Displayed: {},
Quads Displayed Last Frame: {},
Average Quads Displayed over 100 frames: {},
Note: "Rendered" means that widget's render system was ran not that
it was visible on screen. "Displayed" means shown on screen.
=====================================================================
"#,
                metrics.get_widgets_rendered(),
                metrics.get_widgets_rendered_since_last_frame(),
                metrics.get_average_widgets_rendered_per_frame(),
                metrics.get_quads_displayed(),
                metrics.get_quads_displayed_since_last_frame(),
                metrics.get_average_quads_displayed_per_frame()
            );
        }
    }
}
