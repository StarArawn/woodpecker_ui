use std::sync::Arc;

use crate::prelude::*;
use bevy::prelude::*;

/// One column definition for a [`Table`].
#[derive(Reflect, Clone, PartialEq, Debug)]
pub struct TableColumn {
    /// The header label.
    pub header: String,
    /// A fixed width in logical pixels, or `0.0` to have this column grow to fill whatever
    /// space the fixed-width columns don't claim (`flex_grow: 1.0`).
    pub width: f32,
}

impl TableColumn {
    /// A fixed-width column.
    pub fn fixed(header: impl Into<String>, width: f32) -> Self {
        Self {
            header: header.into(),
            width,
        }
    }

    /// A column that grows to fill remaining space.
    pub fn flexible(header: impl Into<String>) -> Self {
        Self {
            header: header.into(),
            width: 0.0,
        }
    }
}

/// One row of a [`Table`] -- plain text cells. `cells.len()` should match the table's
/// `columns.len()`; extra cells are ignored and missing ones render blank.
#[derive(Reflect, Clone, PartialEq, Debug, Default)]
pub struct TableRow {
    /// The row's cell text, in column order.
    pub cells: Vec<String>,
}

impl From<Vec<String>> for TableRow {
    fn from(cells: Vec<String>) -> Self {
        Self { cells }
    }
}

/// Wraps [`Table::rows`] in an `Arc`, so an otherwise-unchanged `Table` compares in O(1)
/// (pointer equality) across renders instead of paying a full O(rows) deep comparison every
/// single frame regardless of whether anything actually changed -- see
/// `diffing::diff_widget_entity`'s reflection-based `PartialEq` check, which runs on every
/// `DiffableProp` component on every widget entity every frame. At real-world table sizes
/// (hundreds of thousands of rows is a realistic ceiling this crate is meant to support via
/// `Self::virtualized`) that comparison alone was measured costing tens of milliseconds *per
/// frame, forever* -- exactly the cost `DiffableProp`'s short-circuiting exists to avoid, and
/// paid regardless of whether virtualization keeps the actually-*spawned* widget count small.
/// Mirrors [`super::virtual_list::VirtualListItemContent`]'s identical reasoning for the same
/// problem.
///
/// Cloning a `TableRows` (e.g. to re-declare an otherwise-unchanged `Table` bundle across
/// renders) is cheap -- it just bumps the `Arc`'s refcount, and compares equal to the original
/// via [`Arc::ptr_eq`]. Construct a genuinely new one (`TableRows::new(rows)`, or `rows.into()`)
/// only when the underlying data actually changes; re-sorting/filtering in place and handing
/// back the *same* `Arc` (e.g. via `Arc::make_mut`) would defeat the point of this type.
#[derive(Debug, Clone, Reflect)]
pub struct TableRows(pub Arc<Vec<TableRow>>);

impl TableRows {
    /// Wraps `rows` for use as [`Table::rows`].
    pub fn new(rows: Vec<TableRow>) -> Self {
        Self(Arc::new(rows))
    }
}

impl Default for TableRows {
    fn default() -> Self {
        Self(Arc::new(Vec::new()))
    }
}

impl PartialEq for TableRows {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl std::ops::Deref for TableRows {
    type Target = Vec<TableRow>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Vec<TableRow>> for TableRows {
    fn from(rows: Vec<TableRow>) -> Self {
        Self::new(rows)
    }
}

impl FromIterator<TableRow> for TableRows {
    fn from_iter<I: IntoIterator<Item = TableRow>>(iter: I) -> Self {
        Self::new(iter.into_iter().collect())
    }
}

/// Fired when a header cell is clicked -- `Table` has no sort state of its own (it doesn't
/// know how to compare your row data), so it just reports which column was clicked and lets
/// the caller re-sort their own data and pass back new `rows`.
#[derive(Debug, Clone, Reflect)]
pub struct TableSort {
    /// The index into `Table::columns` that was clicked.
    pub column_index: usize,
}

/// A collection of styles for [`Table`].
#[derive(Component, Reflect, Clone, PartialEq)]
#[reflect(Component, DiffableProp, PartialEq)]
pub struct TableStyles {
    /// Background of the header row.
    pub header_background: Color,
    /// Background tint applied to every other data row (zebra striping).
    pub zebra_background: Color,
    /// Divider/border color.
    pub border_color: Color,
    /// Corner radius for the table's overall rounded shape (top of the header, bottom of the
    /// last row).
    pub corner_radius: f32,
    /// Horizontal gap between columns.
    pub column_gap: f32,
    /// Horizontal padding inside the header/each row.
    pub row_padding_x: f32,
    /// Vertical padding inside the header/each row.
    pub row_padding_y: f32,
}

impl Default for TableStyles {
    fn default() -> Self {
        Self::from_theme(&Theme::default())
    }
}

impl ThemedStyle for TableStyles {
    fn from_theme(theme: &Theme) -> Self {
        Self {
            header_background: theme.background_mid,
            zebra_background: theme.background_light,
            border_color: theme.border,
            corner_radius: theme.control_radius,
            column_gap: 16.0,
            row_padding_x: 16.0,
            row_padding_y: 10.0,
        }
    }
}

fn default_style() -> WoodpeckerStyle {
    WoodpeckerStyle {
        width: Units::Percentage(100.0),
        flex_direction: WidgetFlexDirection::Column,
        ..Default::default()
    }
}

/// A simple string-cell data table: a header row plus zebra-striped, divided data rows.
/// Cells are plain text -- for rich per-cell content (avatars, dropdowns, buttons), build
/// your own rows from `Element`s using `TableStyles`' values to stay visually consistent.
#[derive(Widget, Component, Reflect, Clone, PartialEq, Default)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(render)]
#[require(
    WoodpeckerStyle = default_style(),
    WidgetChildren,
    TableStyles,
    TableScrollState
)]
pub struct Table {
    /// Column definitions.
    pub columns: Vec<TableColumn>,
    /// Data rows.
    pub rows: TableRows,
    /// When `true`, only spawns the rows actually visible in an *ancestor* `ScrollBox`'s
    /// viewport (plus a small overscan), instead of every row every render -- see
    /// [`compute_virtual_window`](crate::widgets::virtualization::compute_virtual_window).
    /// Requires [`Self::row_height`] to be set and a `ScrollBox`/`ScrollContextProvider`
    /// somewhere above this `Table` in the tree; falls back to rendering every row (as if
    /// this were `false`) when either is missing, so turning it on is always safe.
    pub virtualized: bool,
    /// The fixed height of a single data row, in logical pixels -- required by
    /// [`Self::virtualized`] to know how many rows fit a given scroll offset without having
    /// laid any of them out yet. Ignored when `virtualized` is `false`.
    pub row_height: Option<f32>,
}

/// Extra rows kept mounted just outside the viewport on either side of a virtualized
/// `Table`, so a fast scroll or keyboard nudge doesn't show a blank flash before the next
/// frame's rows spawn in. A *floor*, not the actual overscan used every frame -- see
/// `render`'s velocity-scaled overscan (mirrors `VirtualList`'s own, same reasoning: a fixed
/// overscan this small is easily outrun by a single frame of fast trackpad/wheel scrolling,
/// forcing a mass despawn/respawn of rows that frame -- expensive on top of merely flickering,
/// for a table that can have hundreds of thousands of rows).
const VIRTUALIZED_OVERSCAN: usize = 4;
/// Upper bound on the velocity-scaled overscan -- caps the one-time cost of an extreme
/// single-frame jump (e.g. dragging the scrollbar thumb straight to the bottom) at mounting
/// this many rows, rather than however many thousands a raw delta might imply.
const VIRTUALIZED_MAX_OVERSCAN: usize = 64;

/// Tracks the previous frame's scroll offset purely to size this frame's virtualized overscan
/// -- see `VIRTUALIZED_OVERSCAN`'s doc comment. Not meant to be read by anything else.
/// Deliberately not `DiffableProp`: see `VirtualListScrollState`'s matching doc comment.
#[derive(Component, Default, Clone, PartialEq)]
pub struct TableScrollState {
    prev_scroll_offset: f32,
}

fn cell_style(width: f32) -> WoodpeckerStyle {
    if width > 0.0 {
        WoodpeckerStyle {
            width: width.into(),
            text_wrap: TextWrap::None,
            ..Default::default()
        }
    } else {
        WoodpeckerStyle {
            flex_grow: 1.0,
            text_wrap: TextWrap::None,
            ..Default::default()
        }
    }
}

fn render(
    current_widget: Res<CurrentWidget>,
    theme: Res<Theme>,
    hooks: Res<HookHelper>,
    scroll_context_query: Query<&ScrollContext>,
    mut query: Query<(
        &Table,
        &TableStyles,
        &mut WidgetChildren,
        &mut TableScrollState,
    )>,
) {
    let Ok((table, table_styles, mut children, mut scroll_state)) = query.get_mut(**current_widget)
    else {
        return;
    };

    // Opt-in and self-falling-back: virtualization only kicks in when the caller both asked
    // for it *and* gave us a row height to window with, and only when there's actually an
    // ancestor `ScrollBox` to read a live scroll position from. Any of those missing just
    // means every row renders, same as `virtualized: false`.
    let virtual_window = table
        .virtualized
        .then_some(table.row_height)
        .flatten()
        .and_then(|row_height| {
            let scroll_context = hooks
                .get_context::<ScrollContext>(*current_widget)
                .and_then(|entity| scroll_context_query.get(entity).ok())?;
            let scroll_offset = (-scroll_context.scroll_y()).max(0.0);
            // See `dynamic_overscan`'s own doc comment for why a fixed overscan flickers
            // (and, at this widget's scale, wastes real spawn/despawn work) under fast
            // scrolling, and why this widens it to cover however far the offset moved.
            let overscan = dynamic_overscan(
                scroll_state.prev_scroll_offset,
                scroll_offset,
                row_height,
                VIRTUALIZED_OVERSCAN,
                VIRTUALIZED_MAX_OVERSCAN,
            );
            scroll_state.prev_scroll_offset = scroll_offset;
            Some(compute_virtual_window(
                scroll_offset,
                scroll_context.viewport_height(),
                row_height,
                table.rows.len(),
                overscan,
            ))
        });
    let row_range = virtual_window
        .map(|window| window.start..window.end)
        .unwrap_or(0..table.rows.len());

    *children = WidgetChildren::default();
    let current_widget_val = *current_widget;

    // Header.
    let mut header_children = WidgetChildren::default();
    for (index, column) in table.columns.iter().enumerate() {
        header_children.add::<Element>((
            Element,
            WoodpeckerStyle {
                font_size: 11.0,
                color: theme.text.with_alpha(0.7),
                ..cell_style(column.width)
            },
            WidgetRender::Text {
                content: column.header.clone(),
            },
            // Each header cell needs its own `Pickable` to be its own click target --
            // pointer events only propagate up to ancestors, never down into children, so a
            // click observer on a cell only fires if the cell itself was hit.
            Pickable::default(),
        ));
        header_children.observe(
            current_widget_val,
            move |_trigger: On<Pointer<Click>>, mut commands: Commands| {
                commands.trigger(Change {
                    target: current_widget_val.0,
                    data: TableSort {
                        column_index: index,
                    },
                });
            },
        );
        header_children.hover_cursor(current_widget_val, SystemCursorIcon::Pointer);
        header_children.add_key(format!("h{index}"));
    }
    children.add::<Element>((
        Element,
        WoodpeckerStyle {
            width: Units::Percentage(100.0),
            background_color: table_styles.header_background,
            border_radius: Corner::all(0.0)
                .top_left(table_styles.corner_radius)
                .top_right(table_styles.corner_radius),
            padding: Edge::all(table_styles.row_padding_y)
                .left(table_styles.row_padding_x)
                .right(table_styles.row_padding_x),
            gap: (table_styles.column_gap.into(), 0.0.into()),
            border: Edge::all(0.0).bottom(1.0),
            border_color: table_styles.border_color,
            ..Default::default()
        },
        WidgetRender::Quad,
        header_children,
    ));
    children.add_key("header");

    // Rows.
    if let Some(window) = virtual_window.filter(|window| window.lead_spacer > 0.0) {
        children.add::<Element>((
            Element,
            WoodpeckerStyle {
                width: Units::Percentage(100.0),
                height: window.lead_spacer.into(),
                ..Default::default()
            },
        ));
        children.add_key("lead_spacer");
    }

    let last_row = table.rows.len().saturating_sub(1);
    for row_index in row_range {
        let row = &table.rows[row_index];
        let is_last = row_index == last_row;
        let stripe = if row_index % 2 == 1 {
            table_styles.zebra_background.with_alpha(0.35)
        } else {
            Color::NONE
        };

        let mut row_children = WidgetChildren::default();
        for (col_index, column) in table.columns.iter().enumerate() {
            let text = row.cells.get(col_index).cloned().unwrap_or_default();
            row_children.add::<Element>((
                Element,
                WoodpeckerStyle {
                    font_size: theme.font_size,
                    color: theme.text,
                    ..cell_style(column.width)
                },
                WidgetRender::Text { content: text },
            ));
            row_children.add_key(format!("c{col_index}"));
        }

        children.add::<Element>((
            Element,
            WoodpeckerStyle {
                width: Units::Percentage(100.0),
                align_items: Some(WidgetAlignItems::Center),
                background_color: stripe,
                border_radius: if is_last {
                    Corner::all(0.0)
                        .bottom_left(table_styles.corner_radius)
                        .bottom_right(table_styles.corner_radius)
                } else {
                    Corner::all(0.0)
                },
                padding: Edge::all(table_styles.row_padding_y)
                    .left(table_styles.row_padding_x)
                    .right(table_styles.row_padding_x),
                gap: (table_styles.column_gap.into(), 0.0.into()),
                border: if is_last {
                    Edge::all(0.0)
                } else {
                    Edge::all(0.0).bottom(1.0)
                },
                border_color: table_styles.border_color,
                ..Default::default()
            },
            WidgetRender::Quad,
            row_children,
        ));
        children.add_key(format!("row{row_index}"));
    }

    if let Some(window) = virtual_window.filter(|window| window.trail_spacer > 0.0) {
        children.add::<Element>((
            Element,
            WoodpeckerStyle {
                width: Units::Percentage(100.0),
                height: window.trail_spacer.into(),
                ..Default::default()
            },
        ));
        children.add_key("trail_spacer");
    }

    children.apply(current_widget.as_parent());
}

#[cfg(test)]
mod tests {
    use bevy::ecs::system::RunSystemOnce;

    use super::*;
    use crate::{entity_mapping::WidgetMapper, hook_helper::HookHelper, ObserverCache};

    fn sample_rows(n: usize) -> Vec<TableRow> {
        (0..n)
            .map(|i| TableRow::from(vec![format!("{i}")]))
            .collect()
    }

    #[test]
    fn table_rows_compares_by_pointer_not_value() {
        let rows = sample_rows(3);
        let a = TableRows::new(rows.clone());
        let b = a.clone();
        let c = TableRows::new(rows);

        assert_eq!(
            a, b,
            "a clone (bumped refcount, same Arc) must compare equal"
        );
        assert_ne!(
            a, c,
            "a fresh TableRows built from equal-value data must NOT compare equal -- \
             equality is by Arc pointer, deliberately, so an unrelated re-render carrying a \
             fresh (even if value-identical) TableRows is still correctly treated as changed"
        );
    }

    /// Runs `Table::render` on `table_entity`, then actually materializes its declared
    /// children (mirroring what `runner::system` does every frame) and returns how many real
    /// child entities were spawned -- header included.
    fn render_and_count_children(world: &mut World, table_entity: Entity) -> usize {
        world.insert_resource(CurrentWidget(table_entity));
        world.run_system_once(render).unwrap();
        world.remove_resource::<CurrentWidget>();

        let mut children = world
            .entity_mut(table_entity)
            .get_mut::<WidgetChildren>()
            .unwrap()
            .clone();
        children.apply(ParentWidget(table_entity));
        children.process_world(world);
        world.entity_mut(table_entity).insert(children);

        world
            .entity(table_entity)
            .get::<Children>()
            .map(|c| c.len())
            .unwrap_or(0)
    }

    fn base_app() -> App {
        let mut app = App::new();
        app.init_resource::<Theme>();
        app.init_resource::<HookHelper>();
        app.world_mut().insert_resource(WidgetMapper::new());
        app.world_mut().insert_resource(ObserverCache::default());
        app.register_widget::<Table>();
        app
    }

    fn spawn_table(app: &mut App, table: Table) -> Entity {
        app.world_mut()
            .spawn((table, TableStyles::default(), WidgetChildren::default()))
            .id()
    }

    /// Attaches `table_entity` under a new ancestor entity that provides `scroll_context` via
    /// the same `HookHelper::use_context` mechanism `ScrollContextProvider` uses, then runs
    /// the real `update_context_helper` system so `HookHelper`'s ancestor-lookup cache
    /// actually sees the new `ChildOf` -- i.e. sets up exactly what `Table::render`'s
    /// `hooks.get_context::<ScrollContext>(..)` call walks.
    fn give_ancestor_scroll_context(
        world: &mut World,
        table_entity: Entity,
        scroll_context: ScrollContext,
    ) {
        let ancestor = world.spawn_empty().id();
        world.entity_mut(table_entity).insert(ChildOf(ancestor));
        world
            .run_system_once(
                move |mut hooks: ResMut<HookHelper>, mut commands: Commands| {
                    hooks.use_context(&mut commands, CurrentWidget(ancestor), scroll_context);
                },
            )
            .unwrap();
        world
            .run_system_once(HookHelper::update_context_helper)
            .unwrap();
    }

    #[test]
    fn non_virtualized_renders_every_row_even_with_a_row_height_set() {
        let mut app = base_app();
        let table_entity = spawn_table(
            &mut app,
            Table {
                columns: vec![TableColumn::flexible("A")],
                rows: sample_rows(10).into(),
                virtualized: false,
                row_height: Some(20.0),
            },
        );

        // header + 10 rows, no spacers.
        assert_eq!(render_and_count_children(app.world_mut(), table_entity), 11);
    }

    #[test]
    fn virtualized_without_row_height_falls_back_to_every_row() {
        let mut app = base_app();
        let table_entity = spawn_table(
            &mut app,
            Table {
                columns: vec![TableColumn::flexible("A")],
                rows: sample_rows(10).into(),
                virtualized: true,
                row_height: None,
            },
        );

        assert_eq!(render_and_count_children(app.world_mut(), table_entity), 11);
    }

    #[test]
    fn virtualized_without_an_ancestor_scroll_context_falls_back_to_every_row() {
        let mut app = base_app();
        let table_entity = spawn_table(
            &mut app,
            Table {
                columns: vec![TableColumn::flexible("A")],
                rows: sample_rows(10).into(),
                virtualized: true,
                row_height: Some(20.0),
            },
        );

        // No ancestor `ScrollContext` was ever set up -- `hooks.get_context` finds nothing.
        assert_eq!(render_and_count_children(app.world_mut(), table_entity), 11);
    }

    #[test]
    fn virtualized_with_an_ancestor_scroll_context_windows_the_rows() {
        let mut app = base_app();
        let table_entity = spawn_table(
            &mut app,
            Table {
                columns: vec![TableColumn::flexible("A")],
                rows: sample_rows(1_000).into(),
                virtualized: true,
                row_height: Some(20.0),
            },
        );

        give_ancestor_scroll_context(
            app.world_mut(),
            table_entity,
            ScrollContext {
                scroll_y: 0.0,
                scrollbox_height: 100.0,
                content_height: 20_000.0,
                ..Default::default()
            },
        );

        // Header + a handful of rows near the top (viewport / row_height, plus overscan) --
        // nowhere near all 1,000 rows.
        let count = render_and_count_children(app.world_mut(), table_entity);
        assert!(
            count < 50,
            "expected only the visible window to render, got {count} children"
        );
        assert!(count > 1, "expected at least the header plus some rows");
    }
}
