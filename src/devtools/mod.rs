//! A runtime, in-process widget inspector, entirely behind the `devtools` Cargo feature and
//! never pulled in by [`crate::WoodpeckerUIPlugin`] itself.
//!
//! Add [`WoodpeckerDevtoolsPlugin`] explicitly, and declare exactly one
//! `.with_child::<DevtoolsRoot>(DevtoolsRoot)` in your own app's root children (see
//! [`DevtoolsRoot`]'s doc comment for why the plugin can't add that child for you). Press the
//! toggle key (`F12` by default) to open a docked panel showing the live widget tree; use the
//! "Pick" button to click any on-screen widget and select it; the panel then shows a
//! reflection-based dump of its style/layout/state, plus basic render metrics.
//!
//! Deliberately purpose-built for this crate's own widget model, not a generic ECS browser

use bevy::prelude::*;

use crate::{prelude::*, WidgetRegisterExt};

mod dump;
mod highlight;
mod metrics_readout;
mod panel;
mod pick;
mod style_editor;
mod tree_walk;

pub use panel::DevtoolsRoot;

/// Interactive inspector state -- driven by input systems (the toggle key, the pick-mode
/// button, click-to-select), consumed reactively by [`DevtoolsRoot`]'s render via
/// `WatchedResource<DevtoolsState>`, exactly like `ToastViewport` watches `ToastQueue`.
///
/// `selected` is a plain `Entity` handle -- there's no stable cross-frame widget ID in this
/// crate (see `WidgetMapper`), so a selection can go stale if the inspected app re-renders and
/// despawns/respawns that entity. Every reader treats a missing entity as "no selection"
/// rather than panicking.
///
/// Deliberately does NOT include the live pick-mode hover target -- see [`DevtoolsHover`] for
/// why that's a separate, non-reactive resource instead of a field here.
#[derive(Resource, Reflect, Clone, PartialEq, Default)]
pub struct DevtoolsState {
    /// Is the panel open?
    pub open: bool,
    /// Is "click a widget on screen to select it" armed?
    pub pick_mode: bool,
    /// The currently inspected entity, if any.
    pub selected: Option<Entity>,
    /// The row-key (`"{target}-{field}"`, see `style_editor`) of the color field whose popover
    /// is currently open, if any. At most one at a time -- opening a new one implicitly closes
    /// whatever was open before. Changes rarely (a discrete click), unlike `DevtoolsHover`, so
    /// it's fine for this to live on the reactively-rebuilt `DevtoolsState`.
    pub(crate) open_color_field: Option<String>,
}

/// The widget currently under the cursor while `DevtoolsState::pick_mode` is armed --
/// deliberately NOT a field on `DevtoolsState`. It changes on essentially every frame the
/// mouse moves during pick mode; if it lived on the `WatchedResource`-driven `DevtoolsState`
/// instead, every mouse-move would force a full panel rebuild (title bar, tree, every
/// `NumberInput`/`ColorPicker` row) for the whole duration of picking -- exactly the kind of
/// churn that made clicks flaky elsewhere in the panel while picking was in use. Read directly
/// by `tree_walk::refresh_snapshot_system` (already throttled) for the highlight overlay,
/// bypassing the reactive rebuild path entirely, the same way `metrics_readout` does.
#[derive(Resource, Default, Clone, Copy)]
pub(crate) struct DevtoolsHover(pub(crate) Option<Entity>);

/// A point-in-time snapshot of what the panel displays, recomputed on a throttle by
/// [`tree_walk::refresh_snapshot_system`] and consumed reactively by [`DevtoolsRoot`]'s render
/// the same way as [`DevtoolsState`].
///
/// Deliberately does NOT include render metrics -- those change essentially every frame, and
/// including them here would force the whole panel to reconcile that often too (see
/// `metrics_readout`, which updates its own text element directly instead).
#[derive(Resource, Reflect, Clone, PartialEq, Default)]
pub(crate) struct DevtoolsSnapshot {
    pub(crate) tree: Vec<TreeNode>,
    pub(crate) detail_text: String,
    /// (position, size) of the selected/hovered widget's live `WidgetLayout`, in
    /// virtual-viewport space -- the same space `WoodpeckerStyle`'s `Fixed` positioning
    /// resolves in, so it can be copied straight into the highlight quad's style.
    pub(crate) highlight: Option<(Vec2, Vec2)>,
}

#[derive(Resource, Clone, Copy)]
pub(crate) struct DevtoolsToggleKey(pub(crate) KeyCode);

/// How often [`tree_walk::refresh_snapshot_system`] re-walks the tree and re-dumps the
/// selection while the panel is open. A human-paced tool doesn't need per-frame refresh, and
/// re-walking a widget-rich app's whole tree every frame would be wasted churn.
#[derive(Resource)]
pub(crate) struct DevtoolsRefreshTimer(pub(crate) Timer);

impl Default for DevtoolsRefreshTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(0.25, TimerMode::Repeating))
    }
}

/// Adds the runtime widget inspector. Never pulled in by [`crate::WoodpeckerUIPlugin`] itself
/// -- add it explicitly, alongside a single `.with_child::<DevtoolsRoot>(DevtoolsRoot)` in
/// your own app's root children.
pub struct WoodpeckerDevtoolsPlugin {
    /// The key that toggles the panel open/closed. Defaults to `F12`. Some browsers intercept
    /// `F12` for their own devtools before it reaches the page -- rebind this for WASM builds
    /// if that's a problem for you.
    pub toggle_key: KeyCode,
}

impl Default for WoodpeckerDevtoolsPlugin {
    fn default() -> Self {
        Self {
            toggle_key: KeyCode::F12,
        }
    }
}

impl Plugin for WoodpeckerDevtoolsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(DevtoolsToggleKey(self.toggle_key))
            .init_resource::<DevtoolsState>()
            .init_resource::<DevtoolsHover>()
            .init_resource::<DevtoolsSnapshot>()
            .init_resource::<DevtoolsRefreshTimer>()
            .register_watched_resource::<DevtoolsState>()
            .register_watched_resource::<DevtoolsSnapshot>()
            .register_widget::<DevtoolsRoot>()
            .add_systems(
                Update,
                (
                    toggle_system,
                    pick::click_to_select_system,
                    tree_walk::refresh_snapshot_system,
                    metrics_readout::refresh_metrics_text_system,
                )
                    .chain()
                    .after(crate::runner::system),
            );
    }
}

fn toggle_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    toggle_key: Res<DevtoolsToggleKey>,
    mut state: ResMut<DevtoolsState>,
    mut hover: ResMut<DevtoolsHover>,
) {
    if !keyboard.just_pressed(toggle_key.0) {
        return;
    }
    state.open = !state.open;
    if !state.open {
        state.pick_mode = false;
        hover.0 = None;
    }
}
