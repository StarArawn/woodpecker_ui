use bevy::prelude::*;

/// A single grid track's sizing, used for `GridTemplate::columns`/`rows`/`auto_columns`/
/// `auto_rows`. Doesn't yet support CSS Grid's `repeat()`/`minmax()`/`fit-content()`
/// composition -- each variant maps to a single fixed min/max pair, which covers ordinary
/// fixed, percentage, `fr`, and auto tracks.
#[derive(Reflect, Debug, Clone, Copy, PartialEq, Default)]
pub enum GridTrackSize {
    /// A fixed size, in pixels.
    Pixels(f32),
    /// A percentage of the grid container's size (0.0..=100.0).
    Percentage(f32),
    /// A fraction of the remaining free space (CSS `fr` unit), e.g. `Fraction(1.0)` for `1fr`.
    Fraction(f32),
    /// Sized to fit its content, no larger than necessary.
    #[default]
    Auto,
    /// Sized to the track's content's minimum width/height.
    MinContent,
    /// Sized to the track's content's maximum width/height.
    MaxContent,
}

impl From<GridTrackSize> for taffy::TrackSizingFunction {
    fn from(val: GridTrackSize) -> Self {
        match val {
            GridTrackSize::Pixels(px) => taffy::MinMax {
                min: taffy::MinTrackSizingFunction::length(px),
                max: taffy::MaxTrackSizingFunction::length(px),
            },
            GridTrackSize::Percentage(pct) => taffy::MinMax {
                min: taffy::MinTrackSizingFunction::percent(pct / 100.0),
                max: taffy::MaxTrackSizingFunction::percent(pct / 100.0),
            },
            // Matches CSS's own expansion of a bare `<flex>` track size into `minmax(auto, <flex>)`.
            GridTrackSize::Fraction(fr) => taffy::MinMax {
                min: taffy::MinTrackSizingFunction::auto(),
                max: taffy::MaxTrackSizingFunction::fr(fr),
            },
            GridTrackSize::Auto => taffy::MinMax {
                min: taffy::MinTrackSizingFunction::auto(),
                max: taffy::MaxTrackSizingFunction::auto(),
            },
            GridTrackSize::MinContent => taffy::MinMax {
                min: taffy::MinTrackSizingFunction::min_content(),
                max: taffy::MaxTrackSizingFunction::min_content(),
            },
            GridTrackSize::MaxContent => taffy::MinMax {
                min: taffy::MinTrackSizingFunction::max_content(),
                max: taffy::MaxTrackSizingFunction::max_content(),
            },
        }
    }
}

impl<S: taffy::CheapCloneStr> From<GridTrackSize> for taffy::GridTemplateComponent<S> {
    fn from(val: GridTrackSize) -> Self {
        taffy::GridTemplateComponent::Single(val.into())
    }
}

/// Where a widget is placed along one grid axis (`WoodpeckerStyle::grid_row`/`grid_column`).
/// Named grid lines aren't supported -- lines are addressed by their 1-based index (matching
/// CSS; negative counts from the end of the explicit grid).
#[derive(Reflect, Debug, Clone, Copy, PartialEq, Default)]
pub enum WidgetGridPlacement {
    /// Placed by the auto-placement algorithm.
    #[default]
    Auto,
    /// Placed starting at the given 1-based grid line index (negative counts from the end).
    Line(i16),
    /// Spans the given number of tracks from wherever the auto-placement algorithm puts it.
    Span(u16),
}

impl From<WidgetGridPlacement> for taffy::GridPlacement {
    fn from(val: WidgetGridPlacement) -> Self {
        match val {
            WidgetGridPlacement::Auto => taffy::GridPlacement::Auto,
            WidgetGridPlacement::Line(line) => taffy::GridPlacement::Line(line.into()),
            WidgetGridPlacement::Span(span) => taffy::GridPlacement::Span(span),
        }
    }
}

/// Controls how the auto-placement algorithm fills the grid with items that don't have an
/// explicit `grid_row`/`grid_column` placement.
#[derive(Reflect, Debug, Clone, Copy, PartialEq, Default)]
pub enum WidgetGridAutoFlow {
    /// Fill each row in turn, adding new rows as necessary.
    #[default]
    Row,
    /// Fill each column in turn, adding new columns as necessary.
    Column,
    /// `Row`, plus the dense packing algorithm (backfills earlier gaps where possible).
    RowDense,
    /// `Column`, plus the dense packing algorithm (backfills earlier gaps where possible).
    ColumnDense,
}

impl From<WidgetGridAutoFlow> for taffy::GridAutoFlow {
    fn from(val: WidgetGridAutoFlow) -> Self {
        match val {
            WidgetGridAutoFlow::Row => taffy::GridAutoFlow::Row,
            WidgetGridAutoFlow::Column => taffy::GridAutoFlow::Column,
            WidgetGridAutoFlow::RowDense => taffy::GridAutoFlow::RowDense,
            WidgetGridAutoFlow::ColumnDense => taffy::GridAutoFlow::ColumnDense,
        }
    }
}

/// Explicit grid track definitions for a container whose `WoodpeckerStyle::display` is
/// [`super::WidgetDisplay::Grid`] -- add this alongside `WoodpeckerStyle` on the same entity.
///
/// A separate component rather than fields on [`super::WoodpeckerStyle`] itself: track lists
/// are unbounded (`Vec`), and `WoodpeckerStyle` is `Copy` -- cloned throughout the render and
/// layout hot path -- so it can't hold heap-allocated fields. A widget that isn't a grid
/// container simply omits this component.
#[derive(Component, Reflect, Debug, Clone, PartialEq, Default)]
#[reflect(Component)]
pub struct GridTemplate {
    /// Track sizes for the explicit grid columns, left to right.
    pub columns: Vec<GridTrackSize>,
    /// Track sizes for the explicit grid rows, top to bottom.
    pub rows: Vec<GridTrackSize>,
    /// Track sizing for auto-generated (implicit) grid columns, beyond the explicit ones.
    pub auto_columns: Vec<GridTrackSize>,
    /// Track sizing for auto-generated (implicit) grid rows, beyond the explicit ones.
    pub auto_rows: Vec<GridTrackSize>,
    /// How the auto-placement algorithm fills items with no explicit grid placement.
    pub auto_flow: WidgetGridAutoFlow,
}

impl GridTemplate {
    /// Overlays this template's tracks onto an already-built `taffy::Style` (which already
    /// carries the item-placement fields from `WoodpeckerStyle::grid_row`/`grid_column`).
    pub(crate) fn apply(&self, style: &mut taffy::Style) {
        style.grid_template_columns = self.columns.iter().map(|t| (*t).into()).collect();
        style.grid_template_rows = self.rows.iter().map(|t| (*t).into()).collect();
        style.grid_auto_columns = self.auto_columns.iter().map(|t| (*t).into()).collect();
        style.grid_auto_rows = self.auto_rows.iter().map(|t| (*t).into()).collect();
        style.grid_auto_flow = self.auto_flow.into();
    }
}
