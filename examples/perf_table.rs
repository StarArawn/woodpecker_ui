//! Perf harness for the layout skip-if-clean fast path and `Table` virtualization.
//!
//! Spawns a 5,000-row `Table` (virtualized by default) and otherwise does nothing -- no
//! widget re-renders on its own, no `WoodpeckerStyle` changes on its own. Before the layout
//! skip-if-clean fast path, `layout::system::run` walked and re-`set_style`'d every layout
//! node on *every single frame* regardless of whether anything changed; after it, idle
//! frames cost next to nothing. Before `Table::virtualized`, all 5,000 rows (20,000+ cells)
//! were spawned and laid out every render, whether they were on screen or not; with it, only
//! the rows actually inside the `ScrollBox`'s viewport (plus a small overscan) exist at all.
//!
//! Press `V` to toggle `Table::virtualized` at runtime and compare.
//!
//! Run with `--features metrics` to have Woodpecker UI log `WidgetMetrics` (widgets
//! rendered/frame, quads displayed/frame) to the console every 5 seconds:
//!
//! ```sh
//! cargo run --example perf_table --features metrics
//! ```
//!
//! With virtualization on, "Total Quads Displayed" settles at roughly what fits in the
//! viewport rather than growing with all 5,000 rows; toggle it off with `V` and the next
//! metrics print jumps up as every row spawns.

use bevy::prelude::*;
use woodpecker_ui::prelude::*;

const ROW_COUNT: usize = 500_000;
/// Approximates a data row's real rendered height (`theme.font_size` plus
/// `TableStyles::row_padding_y` top and bottom) closely enough for windowing purposes --
/// `VIRTUALIZED_OVERSCAN` inside `Table` absorbs the small remaining error.
const ROW_HEIGHT: f32 = 34.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WoodpeckerUIPlugin::default())
        .add_systems(Startup, startup)
        .add_systems(Update, toggle_virtualization)
        .run();
}

fn startup(mut commands: Commands, mut ui_context: ResMut<WoodpeckerContext>) {
    commands.spawn((Camera2d, WoodpeckerView));

    let columns = vec![
        TableColumn::fixed("Id", 80.0),
        TableColumn::flexible("Name"),
        TableColumn::fixed("Status", 120.0),
        TableColumn::fixed("Score", 100.0),
    ];

    let rows = (0..ROW_COUNT)
        .map(|i| {
            TableRow::from(vec![
                format!("{i}"),
                format!("Widget {i}"),
                if i % 3 == 0 {
                    "Active".into()
                } else {
                    "Idle".into()
                },
                format!("{:.2}", (i as f32 * 1.37) % 100.0),
            ])
        })
        .collect();

    let root_widget = ui_context.spawn_root(&mut commands);
    commands.entity(*root_widget).insert(
        WidgetChildren::default().with_child::<ScrollContextProvider>((
            ScrollContextProvider::default(),
            WoodpeckerStyle {
                width: Units::Percentage(100.0),
                height: Units::Percentage(100.0),
                ..Default::default()
            },
            WidgetChildren::default().with_child::<ScrollBox>((
                ScrollBox::default(),
                PassedChildren(WidgetChildren::default().with_child::<Table>(Table {
                    columns,
                    rows,
                    virtualized: true,
                    row_height: Some(ROW_HEIGHT),
                })),
            )),
        )),
    );
}

fn toggle_virtualization(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut table_query: Query<&mut Table>,
) {
    if !keyboard_input.just_pressed(KeyCode::KeyV) {
        return;
    }
    for mut table in table_query.iter_mut() {
        table.virtualized = !table.virtualized;
        info!("Table::virtualized = {}", table.virtualized);
    }
}
