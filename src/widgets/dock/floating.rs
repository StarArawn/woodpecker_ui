use bevy::prelude::*;

use crate::prelude::*;

use super::panels::DockPanelRegistry;
use super::tab_bar::DockDragState;
use super::tree::{DockNode, DockTree, NodePath, PanelId};
use super::tree_render::DockRenderCtx;

/// Identifies one [`FloatingDockWindow`] within [`DockFloating::windows`], stable for the
/// window's lifetime (not reused after it's closed/redocked).
#[derive(Reflect, Clone, Copy, PartialEq, Eq, Debug)]
pub struct FloatingWindowId(pub u32);

/// One panel currently floating outside the main dock tree, in its own window.
#[derive(Reflect, Clone, PartialEq, Debug)]
pub struct FloatingDockWindow {
    /// This window's own stable identity within [`DockFloating::windows`].
    pub id: FloatingWindowId,
    /// The panel this window hosts.
    pub panel: PanelId,
    /// The window's current on-screen position.
    pub position: Vec2,
}

/// Panels dragged out of a [`super::DockTree`] entirely, each in its own floating
/// [`WoodpeckerWindow`]. A sibling context to `DockTree` on the same [`super::DockArea`] (both
/// seeded via `hooks.use_own_context` on the same entity), rather than a field on `DockTree`
/// itself -- floating windows aren't part of the tree's split/tab structure at all.
#[derive(Component, Reflect, Clone, PartialEq, Default, Debug)]
#[reflect(Component, DiffableProp, PartialEq)]
pub struct DockFloating {
    /// The currently-floating windows.
    pub windows: Vec<FloatingDockWindow>,
    next_id: u32,
}

impl DockFloating {
    /// Adds `panel` as a new floating window at `position`.
    pub fn push(&mut self, panel: PanelId, position: Vec2) {
        let id = FloatingWindowId(self.next_id);
        self.next_id += 1;
        self.windows.push(FloatingDockWindow {
            id,
            panel,
            position,
        });
    }

    /// Removes the floating window `id`, if still present.
    pub fn remove(&mut self, id: FloatingWindowId) {
        self.windows.retain(|w| w.id != id);
    }
}

/// Wires a [`Pointer<DragEnd>`] observer onto the last-added child (mirroring every other
/// `WidgetChildren::self_observe` call in this crate) that's a pure no-op whenever the drag it's
/// attached to ends without having landed on a valid drop target: `panel` simply stays wherever
/// it already was, since nothing ever removed it from [`DockTree`] in the first place (every
/// `on_drop` handler bails out before touching the tree at all when its own `valid` check fails
/// -- see `drop_zone::render`'s/`tab_bar::render`'s own `on_drop` closures). This used to float
/// the panel into its own window instead; changed on explicit user feedback that an imprecise
/// drop landing just outside a target shouldn't spawn a stray floating window -- reverting
/// (doing nothing) reads as much more forgiving than "oops, now it's a window."
///
/// Uses `self_observe`, not `observe`, so it always targets the header's own entity regardless
/// of whether the caller has already declared other children by the time this runs (`observe`
/// would silently misattach to whichever child was added last -- see `header_render`'s own
/// comment on the exact bug that produced).
///
/// Kept as a real (if now trivial) function/observer, not deleted outright, purely for this one
/// log line -- genuinely useful signal while this feature is still this fresh, distinguishing
/// "the drag was consumed by a drop handler" from "it reverted" at a glance in the console.
pub(super) fn attach_revert_on_unconsumed_drag(
    children: &mut WidgetChildren,
    spawn_location: CurrentWidget,
    drag_state_entity: Entity,
    panel: PanelId,
) {
    children.self_observe(
        spawn_location,
        move |_trigger: On<Pointer<DragEnd>>, drag_state_query: Query<&DockDragState>| {
            let consumed = drag_state_query
                .get(drag_state_entity)
                .is_ok_and(|s| s.consumed);
            if !consumed {
                info!(
                    "dock: drag ended without a valid drop -- panel={:?} reverts to its original \
                     spot",
                    panel.0
                );
            }
        },
    );
}

fn floating_window_style() -> WoodpeckerStyle {
    WoodpeckerStyle {
        min_width: 240.0.into(),
        min_height: 160.0.into(),
        ..WoodpeckerWindow::default().window_styles
    }
}

/// Appends one [`WoodpeckerWindow`] child to `children` per entry in `floating`, each hosting
/// its panel's own content plus a redock button in place of the default title text -- a button,
/// not title-bar-drag-to-redock, since the latter would mean either forking `WoodpeckerWindow`'s
/// own drag observers or building bespoke lighter window chrome, out of proportion to this pass.
/// `WoodpeckerWindow` portals itself to `OverlayRoot` internally, so these are declared as
/// perfectly ordinary children here regardless of where `DockArea` sits in the wider tree.
pub(super) fn render_floating_windows(
    children: &mut WidgetChildren,
    floating: &DockFloating,
    registry: &DockPanelRegistry,
    floating_entity: Entity,
    icon_font: &IconFont,
    ctx: DockRenderCtx,
) {
    for entry in &floating.windows {
        let Some(def) = registry.get(&entry.panel) else {
            continue;
        };

        let title = def.title.clone();
        let panel = entry.panel.clone();
        let id = entry.id;
        let tree_entity = ctx.tree_entity;

        let title_row = WidgetChildren::default()
            .with_child::<Element>((
                Element,
                WoodpeckerStyle {
                    flex_grow: 1.0,
                    font_size: Theme::default().font_size,
                    text_wrap: TextWrap::None,
                    ..Default::default()
                },
                WidgetRender::Text { content: title },
            ))
            .with_key("label")
            .with_child::<Element>((
                Element,
                WoodpeckerStyle {
                    margin: Edge::all(0.0).left(8.0),
                    font_size: Theme::default().font_size,
                    font: Some(icon_font.0.id()),
                    ..Default::default()
                },
                Pickable::default(),
                WidgetRender::Text {
                    content: icons::ARROWS_IN.into(),
                },
            ))
            .with_observe(
                ctx.current_widget,
                move |mut trigger: On<Pointer<Click>>,
                      mut tree_query: Query<&mut DockTree>,
                      mut floating_query: Query<&mut DockFloating>| {
                    info!("dock: redock button clicked panel={:?}", panel.0);
                    trigger.propagate(false);
                    let Ok(mut floating) = floating_query.get_mut(floating_entity) else {
                        info!("dock: redock click -- DockFloating query failed");
                        return;
                    };
                    floating.remove(id);
                    let Ok(mut tree) = tree_query.get_mut(tree_entity) else {
                        info!("dock: redock click -- DockTree query failed");
                        return;
                    };
                    if let Some(target) = tree.last_focused.clone() {
                        info!("dock: redock click -- inserting at last_focused={target:?}");
                        tree.insert_center(&target, panel.clone());
                    } else if let Some(DockNode::Tabs { .. }) = tree.node_at(&NodePath::root()) {
                        info!("dock: redock click -- inserting at root fallback");
                        tree.insert_center(&NodePath::root(), panel.clone());
                    } else {
                        info!("dock: redock click -- no valid target found, panel lost!");
                    }
                },
            );

        children.add::<WoodpeckerWindow>((
            WoodpeckerWindow {
                title: def.title.clone(),
                initial_position: entry.position,
                window_styles: floating_window_style(),
                ..Default::default()
            },
            TitleChildren(title_row),
            PassedChildren((def.factory)(entry.panel.clone())),
        ));
        children.add_key(format!("float_{}", id.0));
    }
}
