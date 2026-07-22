use bevy::prelude::*;

use crate::{metrics::WidgetMetrics, prelude::WidgetRender};

use super::DevtoolsState;

/// Marks the panel's metrics text `Element`, found and updated directly by
/// [`refresh_metrics_text_system`] -- deliberately bypasses `DevtoolsRoot`'s reactive rebuild
/// (`WatchedResource<DevtoolsSnapshot>`) entirely. Render-metrics counters change on
/// essentially every frame; routing them through the same snapshot that drives the whole
/// panel's reconciliation would force a full rebuild that often, for a change with nothing to
/// do with the style editor's `NumberInput`/`ColorPicker` rows sitting right next to it.
#[derive(Component, Clone)]
pub(crate) struct MetricsTextMarker;

pub(crate) fn format_metrics(metrics: &WidgetMetrics) -> String {
    format!(
        "widgets/frame: {} (avg {:.1} over 100 frames) | quads: {} (avg {:.1})",
        metrics.get_widgets_rendered_since_last_frame(),
        metrics.get_average_widgets_rendered_per_frame(),
        metrics.get_quads_displayed_since_last_frame(),
        metrics.get_average_quads_displayed_per_frame(),
    )
}

pub(crate) fn refresh_metrics_text_system(
    state: Res<DevtoolsState>,
    metrics: Res<WidgetMetrics>,
    mut query: Query<&mut WidgetRender, With<MetricsTextMarker>>,
) {
    if !state.open {
        return;
    }
    let Ok(mut render) = query.single_mut() else {
        return;
    };
    let content = format_metrics(&metrics);
    if let WidgetRender::Text { content: current } = &mut *render {
        if *current != content {
            *current = content;
        }
    }
}
