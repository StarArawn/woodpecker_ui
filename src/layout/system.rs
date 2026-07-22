use bevy::{
    ecs::{entity::EntityHashSet, system::SystemParam},
    platform::collections::HashMap,
    prelude::*,
};
use bevy_trait_query::One;
use taffy::Layout;

use crate::{
    context::{Widget, WoodpeckerContext},
    font::FontManager,
    hook_helper::StateMarker,
    portal::{Portal, SkipPortalLint},
    prelude::{GridTemplate, PreviousWidget, WidgetDisplay, WidgetPosition, WidgetRender},
    styles::{Edge, Units},
    svg::SvgAsset,
    DefaultFont,
};

use super::{measure::LayoutMeasure, UiLayout, WoodpeckerStyle};

#[derive(Debug, Copy, Clone, Reflect, Default)]
pub struct ReflectedLayout {
    /// The z value of the node.
    /// This can be adjusted by the user to render nodes ontop of nodes
    /// in the tree regardless of order.
    pub z: u32,
    /// The relative ordering of the node
    ///
    /// Nodes with a higher order should be rendered on top of those with a lower order.
    /// This is effectively a topological sort of each tree.
    pub order: u32,
    /// The top-left corner of the node
    pub location: Vec2,
    /// The width and height of the node
    pub size: Vec2,
    /// The width and height of the content inside the node. This may be larger than the size of the node in the case of
    /// overflowing content and is useful for computing a "scroll width/height" for scrollable nodes
    pub content_size: Vec2,
    /// The size of the scrollbars in each dimension. If there is no scrollbar then the size will be zero.
    pub scrollbar_size: Vec2,
    /// The size of the borders of the node
    pub border: Edge,
    /// The size of the padding of the node
    pub padding: Edge,
}

impl From<&Layout> for ReflectedLayout {
    fn from(value: &Layout) -> Self {
        Self {
            z: 0,
            order: 0,
            location: Vec2::new(value.location.x, value.location.y),
            size: Vec2::new(value.size.width, value.size.height),
            content_size: Vec2::new(value.content_size.width, value.content_size.height),
            scrollbar_size: Vec2::new(value.scrollbar_size.width, value.scrollbar_size.height),
            border: Edge::new(
                value.border.top,
                value.border.right,
                value.border.bottom,
                value.border.left,
            ),
            padding: Edge::new(
                value.padding.top,
                value.padding.right,
                value.padding.bottom,
                value.padding.left,
            ),
        }
    }
}

/// A widget's layout
/// This is built by taffy and included as a component on
/// your widgets automatically when taffy computes layout logic.
#[derive(Component, Debug, Clone, Copy, Deref, DerefMut, Reflect, Default)]
pub struct WidgetLayout(pub ReflectedLayout);

impl WidgetLayout {
    /// The position of the widget in pixels
    pub fn position(&self) -> Vec2 {
        self.location
    }

    /// The width of the layout in pixels
    pub fn width(&self) -> f32 {
        self.0.size.x
    }

    /// The height of the layout in pixels
    pub fn height(&self) -> f32 {
        self.0.size.y
    }

    /// The content width of the layout in pixels
    ///
    /// Not to be confused with width or height this measurement is the amount of space
    /// the children take up.
    pub fn content_width(&self) -> f32 {
        self.0.content_size.x
    }

    /// The content height of the layout in pixels
    ///
    /// Not to be confused with width or height this measurement is the amount of space
    /// the children take up.
    pub fn content_height(&self) -> f32 {
        self.0.content_size.y
    }
}

// TODO: Add more here..
fn layout_equality(layout_a: &ReflectedLayout, layout_b: &ReflectedLayout) -> bool {
    layout_a.size == layout_b.size
        && layout_a.location == layout_b.location
        && layout_a.content_size == layout_b.content_size
}

impl std::cmp::PartialEq<WidgetLayout> for WidgetPreviousLayout {
    fn eq(&self, other: &WidgetLayout) -> bool {
        layout_equality(self, other)
    }
}

impl PartialEq for WidgetLayout {
    fn eq(&self, other: &Self) -> bool {
        layout_equality(self, other)
    }
}

/// The previous layout from the last frame.
/// Useful in some cases to see if a widget's layout has
/// changed.
#[derive(Component, Debug, Clone, Copy, Deref, DerefMut, Reflect)]
pub struct WidgetPreviousLayout(pub ReflectedLayout);

impl PartialEq for WidgetPreviousLayout {
    fn eq(&self, other: &Self) -> bool {
        layout_equality(self, other)
    }
}

/// Opt-in marker: a widget adds this (usually via `#[require(...)]`) to say "also re-render
/// me whenever MY OWN `WidgetLayout` changes."
///
/// `WidgetLayout` can't be a generic [`crate::diffable_prop::DiffableProp`] globally since it's
/// recomputed every frame on nearly every widget, which would force the whole tree to
/// re-render on any layout pass. This is a narrow, per-widget opt-in instead.
#[derive(Component, Default)]
pub struct WatchLayout;

#[derive(Component, Default)]
pub(crate) struct PreviousWatchedLayout(Option<WidgetLayout>);

/// Opt-in marker: this widget establishes a new interactive stacking tier (its own picking
/// `z` and its descendants' are one step above whatever tier it itself sits in), so it and
/// everything nested under it always wins pick priority over anything in an ancestor tier.
///
/// Deliberately keyed off this marker rather than `WidgetRender::Layer`, since `Layer` is
/// also used by the much more common `Clip` (scroll clipping, swatches, etc.), which has
/// nothing to do with interactive stacking order.
#[derive(Component, Default, Clone, Copy)]
pub struct StackingContext;

/// Returns `true` (and updates the snapshot) if `entity` has both [`WatchLayout`] and a
/// [`WidgetLayout`] whose value changed since the last check.
///
/// Ignores a degenerate `(0, 0)` size rather than treating it as a real baseline: a widget
/// hidden under a `display: None` ancestor lays out at zero size every frame, and letting
/// that update the snapshot would make becoming visible again register as a spurious change
/// (visible as a brief flicker/jump for widgets that feed their own layout back into layout,
/// e.g. `ScrollContent`).
pub(crate) fn diff_watched_layout(world: &mut World, entity: Entity) -> bool {
    if world.get::<WatchLayout>(entity).is_none() {
        return false;
    }
    let Some(&current) = world.get::<WidgetLayout>(entity) else {
        return false;
    };
    if current.size == Vec2::ZERO {
        return false;
    }

    let changed = match world.get::<PreviousWatchedLayout>(entity) {
        Some(previous) => previous.0 != Some(current),
        None => true,
    };

    if changed {
        world
            .entity_mut(entity)
            .insert(PreviousWatchedLayout(Some(current)));
    }

    changed
}

#[derive(SystemParam)]
pub(crate) struct LayoutSystemParam<'w, 's> {
    commands: Commands<'w, 's>,
    default_font: Res<'w, DefaultFont>,
    font_manager: ResMut<'w, FontManager>,
    ui_layout: ResMut<'w, UiLayout>,
    query: Query<
        'w,
        's,
        (
            Entity,
            One<&'static dyn Widget>,
            &'static WoodpeckerStyle,
            Option<&'static GridTemplate>,
            Option<&'static ChildOf>,
            Option<&'static Children>,
        ),
        (Without<StateMarker>, Without<PreviousWidget>),
    >,
    state_marker_query: Query<'w, 's, &'static StateMarker>,
    prev_marker_query: Query<'w, 's, &'static PreviousWidget>,
    children_query: Query<
        'w,
        's,
        (Entity, &'static Children, One<&'static dyn Widget>),
        (Changed<Children>, Without<PreviousWidget>),
    >,
    layout_query: Query<'w, 's, &'static WidgetLayout>,
    widget_render: Query<'w, 's, &'static WidgetRender>,
    stacking_context_query: Query<'w, 's, &'static StackingContext>,
    /// Used only by the dev-build lint below (`warn_once!` on a non-portaled `Global(n)` in
    /// the built-in tier range) -- see `traverse_layout_update`'s call site.
    portal_query: Query<'w, 's, (), With<Portal>>,
    /// Also used only by that same lint -- see [`crate::portal::SkipPortalLint`]'s doc comment
    /// for why a non-portaled entity can still legitimately be exempt.
    skip_portal_lint_query: Query<'w, 's, (), With<SkipPortalLint>>,
    context: Res<'w, WoodpeckerContext>,
    image_assets: Res<'w, Assets<Image>>,
    svg_assets: Res<'w, Assets<SvgAsset>>,
    removed_widgets: RemovedComponents<'w, 's, WidgetLayout>,
    /// Non-empty exactly when something a layout pass would actually need to react to
    /// changed this frame -- deliberately *not* filtered to the same `(Without<StateMarker>,
    /// Without<PreviousWidget>)` bounds as `query`: erring toward over-detecting "dirty" is
    /// safe (worst case, one wasted full layout pass), under-detecting it would silently
    /// drop a real layout update.
    dirty_query: Query<
        'w,
        's,
        Entity,
        Or<(
            Changed<WoodpeckerStyle>,
            Changed<GridTemplate>,
            Changed<WidgetRender>,
        )>,
    >,
    /// Deliberately not the full `query` above (with its `Widget`/marker bounds) -- this is
    /// only ever used to look up a tracked text entity's parent for the stale-measurement
    /// check below, and keeping it minimal means it can't itself fail to match a text entity
    /// that the main `query` would.
    child_of_query: Query<'w, 's, &'static ChildOf>,
}

// TODO: Document how layouting works..
pub(crate) fn run(layout_system_param: LayoutSystemParam) {
    let LayoutSystemParam {
        mut commands,
        default_font,
        mut font_manager,
        mut ui_layout,
        state_marker_query,
        query,
        prev_marker_query,
        children_query,
        layout_query,
        widget_render,
        stacking_context_query,
        portal_query,
        skip_portal_lint_query,
        context,
        image_assets,
        svg_assets,
        mut removed_widgets,
        dirty_query,
        child_of_query,
    } = layout_system_param;

    let root_node = context.get_root_widget();
    ui_layout.root_entity = root_node;

    // `RemovedComponents::read()` drains its event cursor, so it can only be consumed once
    // per frame -- collect it up front to both feed the dirty check below and drive the
    // (always-necessary, regardless of whether the rest of this pass runs) taffy cleanup.
    let removed: Vec<Entity> = removed_widgets.read().collect();
    for &entity in &removed {
        ui_layout.remove_child(entity);
    }

    let Ok((width, height)) = query
        .get(root_node)
        .map(|(_, _, style, _, _, _)| (style.width.value_or(1.0), style.height.value_or(1.0)))
    else {
        return;
    };
    let root_size = Vec2::new(width, height);

    // Collected once and reused for two different granularities: `is_empty()` below drives
    // the whole-frame skip-if-clean fast path, and the set itself drives
    // `traverse_upsert_node`'s per-entity gating (Phase 9's incremental `set_style` gating) --
    // only entities actually in this set (plus text entities whose parent resized, tracked
    // separately) pay for a taffy `set_style`/measure this frame.
    let dirty_entities: EntityHashSet = dirty_query.iter().collect();

    // A text entity's *own* `Changed<T>` set (`dirty_entities`) says nothing about whether its
    // *parent's* committed width has since settled to something different than what its
    // current measurement was taken against -- exactly the case `previous_measure_width`/
    // `traverse_upsert_node`'s `parent_width_changed` exist to catch, but only once a walk
    // actually happens. Without this check, a text entity whose parent's final size only
    // becomes known a few frames after both are first created (routine for any multi-level
    // widget composition, e.g. a custom widget wrapping a `WButton`) can get measured once
    // against a wrong transient width, wrap incorrectly, and then never be revisited if
    // nothing *else* stays dirty afterward -- silently stuck wrong until some unrelated change
    // forces a walk again. Checked against `ui_layout.get_layout` (this function's own taffy
    // cache), the same source `parent_width_changed` reads, so both agree on what "settled"
    // means.
    let stale_measured_text =
        ui_layout
            .previous_measure_width
            .iter()
            .any(|(&text_entity, &measured_width)| {
                let Ok(parent) = child_of_query.get(text_entity) else {
                    return false;
                };
                let Some(parent_layout) = ui_layout.get_layout(parent.parent()) else {
                    return false;
                };
                (parent_layout.size.width - measured_width).abs() > 0.01
            });

    // Same idea as `stale_measured_text`, generalized to any `Units::Calc` field on any
    // entity -- see `traverse_upsert_node`'s `calc_stale` for the full reasoning. Without
    // this, the skip-if-clean fast path below could start trusting a frame as idle before
    // `traverse_upsert_node` ever gets a chance to notice a calc-styled entity's parent has
    // since settled to a different size.
    let stale_calc = ui_layout
        .previous_calc_parent_size
        .iter()
        .any(|(&calc_entity, &resolved_against)| {
            let Ok(parent) = child_of_query.get(calc_entity) else {
                return false;
            };
            let Some(parent_layout) = ui_layout.get_layout(parent.parent()) else {
                return false;
            };
            let current = Vec2::new(parent_layout.size.width, parent_layout.size.height);
            (current - resolved_against).length_squared() > 0.0001
        });

    // How many consecutive clean-looking frames to insist on before actually trusting the
    // skip-if-clean fast path below. `dirty_entities`/`children_query`/`removed`/
    // `last_root_size`/`stale_measured_text` each cover a *specific* known way this frame can
    // be dirty, but a freshly spawned or reshuffled subtree (e.g. a virtualized table's rows
    // sliding their window, a popup's content just mounting) can take a couple of frames to
    // fully settle in ways none of those signals individually catch on the very next frame --
    // previously harmless, since the old always-render-every-frame behavior gave it those
    // extra frames "for free" and any single-frame glitch was invisibly overdrawn a moment
    // later. Now that a clean frame can mean "stop rendering entirely", trusting the very
    // first one risks permanently freezing on a still-converging, visibly wrong frame. Staying
    // "dirty" (from this flag's consumers' point of view) for a few extra frames after the
    // last real change is a small, bounded cost that gives that convergence room to finish
    // before the fast path ever engages.
    const SETTLE_FRAMES: u32 = 5;

    // Skip-if-clean fast path: a genuinely idle frame (the common case for dashboards,
    // HUDs, or any UI sitting still between interactions) shouldn't pay for a full O(n)
    // tree walk + taffy layout pass + O(n) copy-back. `dirty_entities`/`children_query` cover
    // any widget prop/state change; `removed` covers despawns; comparing `root_size`
    // against `UiLayout::last_root_size` covers a window resize, which touches no
    // `Changed<T>` we'd otherwise see since it's read fresh from styles every frame rather
    // than mutated; `stale_measured_text`/`stale_calc` cover a text/calc entity whose
    // parent's size settled after the fact (see above).
    let anything_dirty = !dirty_entities.is_empty()
        || !children_query.is_empty()
        || !removed.is_empty()
        || ui_layout.last_root_size != Some(root_size)
        || stale_measured_text
        || stale_calc;

    if anything_dirty {
        ui_layout.settled_frames_in_a_row = 0;
    } else {
        ui_layout.settled_frames_in_a_row = ui_layout.settled_frames_in_a_row.saturating_add(1);
    }

    let take_fast_path = ui_layout.settled_frames_in_a_row > SETTLE_FRAMES;
    ui_layout.dirty_this_frame = !take_fast_path;
    if take_fast_path {
        return;
    }
    ui_layout.last_root_size = Some(root_size);

    // This needs to be in the correct order
    traverse_upsert_node(
        root_node,
        &query,
        &widget_render,
        &default_font,
        &mut font_manager,
        &image_assets,
        &svg_assets,
        &mut ui_layout,
        root_node,
        Vec2::new(1.0, 1.0),
        &dirty_entities,
    );

    for (entity, children, _) in children_query.iter() {
        let normal_children = children
            .iter()
            // We only want to add non-fixed entities as children
            .filter(|child| {
                let Ok((_, _, styles, _, _, _)) = query.get(*child) else {
                    return false;
                };
                !matches!(styles.position, WidgetPosition::Fixed)
            })
            .filter(|child| {
                !state_marker_query.contains(*child) && !prev_marker_query.contains(*child)
            })
            .collect::<Vec<_>>();
        ui_layout.add_children(entity, &normal_children);

        // A `position: Fixed` child is hoisted to the root for layout (so it resolves
        // against the viewport, not its logical parent). That bypasses the parent's own
        // `display: None` collapse, so a hidden Fixed child would otherwise stay fully
        // clickable -- detach it instead whenever the logical parent is hidden.
        let parent_hidden = query
            .get(entity)
            .is_ok_and(|(_, _, styles, _, _, _)| matches!(styles.display, WidgetDisplay::None));
        for child in children {
            let Ok((_, _, styles, _, _, _)) = query.get(*child) else {
                continue;
            };
            if styles.position == WidgetPosition::Fixed {
                if parent_hidden {
                    ui_layout.detach_child(root_node, *child);
                } else {
                    ui_layout.add_child(root_node, *child);
                }
            }
        }
    }

    ui_layout.compute(root_node, root_size);

    // TODO(PERF): Figure out how we can combine traversal and compute together..
    let mut order = 0;
    let mut cache = HashMap::default();
    traverse_layout_update(
        &mut commands,
        root_node,
        &ui_layout,
        &query,
        &stacking_context_query,
        &layout_query,
        &portal_query,
        &skip_portal_lint_query,
        &mut cache,
        &mut order,
        0,
    );
}

/// Orders `children` for paint/pick purposes: entities with no explicit `WidgetZ::Global` sort
/// first, in DOM order; entities with an explicit `WidgetZ::Global(n)` sort after, ascending by
/// `n` (DOM order as the tiebreak within equal `n`).
///
/// This is the ONLY place z-index-driven reordering happens, applied once per parent at every
/// level of the tree (not just at `StackingContext` boundaries) -- which is exactly what makes
/// a descendant's `Global(n)`, however large, only ever competitive against ITS OWN siblings
/// (same immediate parent), never able to "jump the queue" past a grandparent's other children.
/// A `StackingContext`-tagged widget still escapes an ambient sibling group the same way any
/// other explicit `Global(n)` does -- by winning its OWN parent's bucket sort -- it just also
/// happens to bump the z-fallback `layer_depth` for its own descendants (see the call site
/// below). Escaping further up the tree (e.g. a `Dropdown` deeply nested inside a `Clip`d
/// scroll container needing to render above unrelated page content) requires physically
/// relocating the entity -- see `crate::portal`.
///
/// Takes a plain closure rather than a specific `Query` type so both this module's own
/// `traverse_layout_update` and `vello_renderer.rs`'s `traverse_render_tree` (whose queries
/// have different component sets) can share this exact same algorithm instead of maintaining
/// two copies that could silently drift apart.
pub(crate) fn order_children_for_paint(
    children: &[Entity],
    global_z_of: impl Fn(Entity) -> Option<u32>,
) -> Vec<Entity> {
    let mut auto = Vec::new();
    let mut explicit: Vec<(u32, usize, Entity)> = Vec::new();
    for (dom_index, &child) in children.iter().enumerate() {
        match global_z_of(child) {
            None => auto.push(child),
            Some(n) => explicit.push((n, dom_index, child)),
        }
    }
    explicit.sort_by_key(|&(n, dom_index, _)| (n, dom_index));
    auto.into_iter()
        .chain(explicit.into_iter().map(|(_, _, e)| e))
        .collect()
}

fn traverse_layout_update(
    commands: &mut Commands,
    entity: Entity,
    ui_layout: &UiLayout,
    query: &Query<
        (
            Entity,
            One<&dyn Widget>,
            &WoodpeckerStyle,
            Option<&GridTemplate>,
            Option<&ChildOf>,
            Option<&Children>,
        ),
        (Without<StateMarker>, Without<PreviousWidget>),
    >,
    stacking_context_query: &Query<&StackingContext>,
    layout_query: &Query<&WidgetLayout>,
    portal_query: &Query<(), With<Portal>>,
    skip_portal_lint_query: &Query<(), With<SkipPortalLint>>,
    cache: &mut HashMap<Entity, Layout>,
    order: &mut u32,
    layer_depth: u32,
) {
    let Ok((entity, _, styles, _, parent, children)) = query.get(entity) else {
        return;
    };
    if let Some(layout) = ui_layout.get_layout(entity) {
        let mut layout = *layout;
        if let Ok(prev_layout) = layout_query.get(entity) {
            commands
                .entity(entity)
                .insert(WidgetPreviousLayout(prev_layout.0));
        }

        if let Some(parent_layout) = parent.map(|parent| {
            cache
                .get(&parent.parent())
                .unwrap_or(ui_layout.get_layout(parent.parent()).unwrap())
        }) {
            if styles.position != WidgetPosition::Fixed {
                layout.location.x += parent_layout.location.x;
                layout.location.y += parent_layout.location.y;
            }
        }

        cache.insert(entity, layout);
        let mut layout = WidgetLayout((&layout).into());
        layout.order =
            ((*order as i32) + styles.z_index.and_then(|z| z.get_relative()).unwrap_or(0)) as u32;
        // Without an explicit global z-index, fall back to how many `StackingContext`
        // boundaries (e.g. a `Modal`) this entity is nested under, so each nested modal gets
        // a genuine whole-integer Z step instead of relying on the epsilon-scaled `order`
        // counter to separate layers. `order` still disambiguates siblings within one layer.
        layout.z = styles
            .z_index
            .and_then(|z| z.get_global())
            .unwrap_or(layer_depth);
        *order += 1;
        commands.entity(entity).insert(layout);

        // Dev-build-only fingerprint of "someone copied the old escape-to-global-z pattern and
        // needs `.portal()` instead": an explicit `Global(n)` only ever wins against this
        // entity's own immediate siblings (see `order_children_for_paint`) -- it can no
        // longer "jump the queue" past an ancestor's other content the way it could before
        // true CSS-scoped stacking landed. `n` in the built-in tiers' range
        // (`StackingTier::Modal` and up, i.e. this crate's own overlay widgets) on an entity
        // that isn't physically portaled is the exact shape of a copy-pasted escape hatch
        // that will render/pick in the wrong place the moment it's nested inside anything
        // with its own ambient content.
        #[cfg(debug_assertions)]
        if let Some(n) = styles.z_index.and_then(|z| z.get_global()) {
            if n > 50_000
                && !portal_query.contains(entity)
                && !skip_portal_lint_query.contains(entity)
            {
                bevy::log::warn_once!(
                    "Woodpecker UI: entity {entity:?} has WidgetZ::Global({n}), in this \
                     crate's built-in overlay tier range (see StackingTier), but isn't \
                     `.portal()`-ed. A high z_index only wins against its own immediate \
                     siblings -- it no longer escapes an ancestor's ambient content on its \
                     own. If this entity is meant to render on top of everything regardless \
                     of where it's declared, `.portal()` it instead."
                );
            }
        }

        let is_stacking_context = stacking_context_query.contains(entity);
        // Based on this entity's own resolved `layout.z`, not the incoming ambient
        // `layer_depth` -- needed so each `WoodpeckerWindow`'s distinct z_index (bumped on
        // focus/drag via `WindowingContext::shift_to_top`) propagates to its own content,
        // otherwise overlapping windows' controls would all pick at the same depth and tie-
        // break on tree order instead of which window was last brought to front.
        let child_layer_depth = if is_stacking_context {
            layout.z + 1
        } else {
            layout.z
        };

        let Some(children) = children.map(|c| c.iter().collect::<Vec<_>>()) else {
            return;
        };
        let children = order_children_for_paint(&children, |child| {
            query
                .get(child)
                .ok()
                .and_then(|(_, _, styles, ..)| styles.z_index)
                .and_then(|z| z.get_global())
        });

        for child in children.iter() {
            traverse_layout_update(
                commands,
                *child,
                ui_layout,
                query,
                stacking_context_query,
                layout_query,
                portal_query,
                skip_portal_lint_query,
                cache,
                order,
                child_layer_depth,
            );
        }
    }
}

/// A parent width change is only a reason to remeasure for `Text`/`RichText` -- their
/// `LayoutMeasure` is a `Fixed` size baked in ahead of time from `measure_text`'s word-wrap
/// pass against the parent's *current* width, and `Fixed::measure` ignores whatever
/// constraints taffy passes it at compute time, so nothing but re-running this baking step
/// picks up a new wrap width. `Image`/`Svg` don't need the same tracking: their
/// `ImageMeasure` genuinely uses the `available_width`/`available_height` taffy passes it at
/// compute time, and taffy's own per-node cache is keyed on that available space (see
/// `taffy::Cache::get`) -- a parent resize that changes a child's available space is already
/// a cache miss there, so taffy re-invokes the measure closure with fresh constraints on its
/// own, with no help needed from this traversal.
fn measures_via_parent_width(widget_render: &WidgetRender) -> bool {
    matches!(
        widget_render,
        WidgetRender::Text { .. } | WidgetRender::RichText { .. }
    )
}

#[allow(clippy::too_many_arguments)]
fn traverse_upsert_node(
    root_node: Entity,
    query: &Query<
        (
            Entity,
            One<&dyn Widget>,
            &WoodpeckerStyle,
            Option<&GridTemplate>,
            Option<&ChildOf>,
            Option<&Children>,
        ),
        (Without<StateMarker>, Without<PreviousWidget>),
    >,
    query_widget_render: &Query<&WidgetRender>,
    default_font: &DefaultFont,
    font_manager: &mut FontManager,
    image_assets: &Assets<Image>,
    svg_assets: &Assets<SvgAsset>,
    layout: &mut UiLayout,
    current_node: Entity,
    camera_scale: Vec2,
    dirty_entities: &EntityHashSet,
) {
    let Ok((entity, _, styles, grid_template, parent, children)) = query.get(current_node) else {
        return;
    };

    let widget_render = query_widget_render.get(entity).ok();
    // Converted to an owned, `Copy` `WidgetLayout` immediately -- `layout.get_layout(..)`
    // borrows from `layout`, and this value is later used alongside a mutable borrow of
    // `layout.previous_measure_width`, which a live reference into `layout` would conflict
    // with.
    let parent_layout: Option<WidgetLayout> = if let Some(parent_entity) = parent {
        layout.get_layout(parent_entity.parent())
    } else {
        layout.get_layout(root_node)
    }
    .map(|l| WidgetLayout(ReflectedLayout::from(l)));

    // A parent resize touches no `Changed<T>` on *this* entity, so it can't show up in
    // `dirty_entities` -- tracked separately for the one render kind that actually needs it.
    // See `measures_via_parent_width`'s doc comment.
    let parent_width_changed = widget_render.is_some_and(measures_via_parent_width)
        && parent_layout.is_some_and(|parent_layout| {
            let current_width = parent_layout.size.x;
            match layout.previous_measure_width.get(&entity) {
                Some(&previous_width) => (current_width - previous_width).abs() > 0.01,
                None => true,
            }
        });

    // Same class of problem as `parent_width_changed`, generalized to any `Units::Calc`
    // field on any entity (not just text measurement): a `Calc` value's own `WoodpeckerStyle`
    // never itself changes when only the *parent's* committed size does, so without this,
    // an entity that stops being independently dirty gets stuck forever at whatever it
    // resolved to on its own first (often parent-not-yet-computed, `Vec2::ZERO`) frame. Only
    // fires once a real parent layout exists -- the very first, `parent_layout: None` frame
    // is already covered by `dirty_entities` (a fresh spawn), and deliberately isn't recorded
    // into `previous_calc_parent_size` below, so this correctly fires again once the parent's
    // real size lands.
    let calc_parent_size =
        parent_layout.map(|parent_layout| Vec2::new(parent_layout.width(), parent_layout.height()));
    let calc_stale = styles.uses_calc()
        && calc_parent_size.is_some_and(|current| match layout.previous_calc_parent_size.get(&entity) {
            Some(&previous) => (current - previous).length_squared() > 0.0001,
            None => true,
        });

    if dirty_entities.contains(&entity) || parent_width_changed || calc_stale {
        let layout_measure = widget_render.and_then(|widget_render| {
            let parent_layout = parent_layout?;
            // A text entity's very first measurement ever is unbounded (no wrap constraint at
            // all), regardless of the parent's own width mode -- this establishes its true
            // natural single-line size as the foundation everything else builds on. Without
            // this, an `Auto`-width (hug-content) parent -- e.g. any default `WButton`, which
            // deliberately has no `width` override so it hugs its label -- creates a circular
            // dependency: the text measures against the parent's *previous* committed width,
            // but that width is itself derived from the text's own last measurement. If the
            // very first measurement ever happens to land against a too-narrow width (e.g. 0,
            // before the parent has any committed layout at all) and wraps, taffy commits the
            // parent's hug-content size to match that wrapped (narrow) result -- and now the
            // "previous" and "current" parent widths agree on being narrow, so nothing ever
            // detects this as stale (see `stale_measured_text` above, which only catches a
            // parent's width *changing*, not one that's wrong but internally self-consistent).
            // A `Fixed`/percentage-width parent doesn't have this problem the same way: its
            // width doesn't depend on the text, so once its true width is committed,
            // `stale_measured_text` reliably flushes a corrected measurement through. Text
            // that never fits in ANY reasonable width still wraps correctly on the very next
            // measurement, once the parent's true (whether hug-content or fixed) width is
            // known -- only the first, foundational measurement is unbounded.
            let is_first_measurement = measures_via_parent_width(widget_render)
                && !layout.previous_measure_width.contains_key(&entity);
            if measures_via_parent_width(widget_render) {
                layout
                    .previous_measure_width
                    .insert(entity, parent_layout.size.x);
            }
            // `text_wrap: TextWrap::None` also always measures unbounded, on every
            // measurement, not just the first: `text_wrap` otherwise only maps to
            // `OverflowWrap` (how a single too-long *word* breaks, in `measure_text`
            // below) -- it was never actually wired to the width constraint fed to
            // parley, so a `None`-wrapped text still got constrained to (and wrapped
            // within) its parent's current width on every measurement after the first.
            // A single-line, "let it overflow/get clipped" text needs its true natural
            // width every time, the same way the very-first-measurement case above does.
            let unbounded =
                is_first_measurement || styles.text_wrap == crate::styles::TextWrap::None;
            match_render_size(
                font_manager,
                image_assets,
                svg_assets,
                default_font,
                widget_render,
                styles,
                &parent_layout,
                camera_scale,
                unbounded,
            )
        });

        if styles.uses_calc() {
            if let Some(calc_parent_size) = calc_parent_size {
                layout
                    .previous_calc_parent_size
                    .insert(entity, calc_parent_size);
            }
        }
        let resolved_styles = styles.resolve_calc(calc_parent_size.unwrap_or(Vec2::ZERO));
        layout.upsert_node(entity, &resolved_styles, grid_template, layout_measure);
    }

    let Some(children) = children else {
        return;
    };
    for child in children.iter() {
        traverse_upsert_node(
            root_node,
            query,
            query_widget_render,
            default_font,
            font_manager,
            image_assets,
            svg_assets,
            layout,
            child,
            camera_scale,
            dirty_entities,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn match_render_size(
    font_manager: &mut FontManager,
    image_assets: &Assets<Image>,
    svg_assets: &Assets<SvgAsset>,
    default_font: &DefaultFont,
    widget_render: &WidgetRender,
    styles: &WoodpeckerStyle,
    parent_layout: &WidgetLayout,
    camera_scale: Vec2,
    unbounded: bool,
) -> Option<LayoutMeasure> {
    match widget_render {
        WidgetRender::Image { handle } => {
            let image = image_assets.get(handle)?;

            let size = image.size().as_vec2();

            Some(LayoutMeasure::Image(super::measure::ImageMeasure { size }))
        }
        WidgetRender::Svg { handle, .. } => {
            let svg_asset = svg_assets.get(handle)?;

            let size = Vec2::new(svg_asset.width, svg_asset.height);
            Some(LayoutMeasure::Image(super::measure::ImageMeasure { size }))
        }
        WidgetRender::RichText { content } => measure_text(
            &content.text,
            styles,
            font_manager,
            default_font,
            parent_layout,
            camera_scale,
            unbounded,
        ),
        WidgetRender::Text { content } => measure_text(
            content,
            styles,
            font_manager,
            default_font,
            parent_layout,
            camera_scale,
            unbounded,
        ),
        _ => None,
    }
}

/// Resolves `max_width` to a concrete pixel value against `basis` (the parent's committed
/// width), or `None` if it doesn't constrain anything (`Auto`). Delegates `Calc` to
/// [`Units::resolve_calc`] (collapsing it to `Pixels`) rather than re-deriving its formula here,
/// in case this runs before the style's own eager resolution pass has done so already.
fn resolve_max_width(max_width: Units, basis: f32) -> Option<f32> {
    match max_width.resolve_calc(basis) {
        Units::Pixels(px) => Some(px),
        Units::Percentage(pct) => Some(basis * pct / 100.0),
        Units::Auto | Units::Calc { .. } => None,
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn measure_text(
    text: &str,
    styles: &WoodpeckerStyle,
    font_manager: &mut FontManager,
    default_font: &DefaultFont,
    parent_layout: &WidgetLayout,
    camera_scale: Vec2,
    unbounded: bool,
) -> Option<LayoutMeasure> {
    // Measure text
    // TODO: Cache this.
    let mut layout_editor = parley::PlainEditor::new(styles.font_size);
    layout_editor.set_text(text);
    let text_styles = layout_editor.edit_styles();
    text_styles.insert(parley::StyleProperty::LineHeight(
        parley::LineHeight::FontSizeRelative(
            styles
                .line_height
                .map(|lh| styles.font_size / lh)
                .unwrap_or(1.2),
        ),
    ));
    text_styles.insert(parley::StyleProperty::FontStack(parley::FontStack::Single(
        parley::FontFamily::Named(
            font_manager
                .get_family(styles.font.as_ref().unwrap_or(&default_font.0.id()))
                .into(),
        ),
    )));

    text_styles.insert(parley::StyleProperty::OverflowWrap(
        match styles.text_wrap {
            crate::styles::TextWrap::None => parley::OverflowWrap::Normal,
            crate::styles::TextWrap::Glyph => parley::OverflowWrap::Anywhere,
            crate::styles::TextWrap::Word => parley::OverflowWrap::BreakWord,
            crate::styles::TextWrap::WordOrGlyph => parley::OverflowWrap::Anywhere,
        },
    ));
    layout_editor.set_width(if unbounded {
        None
    } else {
        let mut wrap_width = parent_layout.size.x;
        if let Some(max_width) = resolve_max_width(styles.max_width, parent_layout.size.x) {
            wrap_width = wrap_width.min(max_width);
        }
        Some(wrap_width * camera_scale.x)
    });
    let alignment = match styles
        .text_alignment
        .unwrap_or(crate::font::TextAlign::Left)
    {
        crate::font::TextAlign::Left => parley::Alignment::Left,
        crate::font::TextAlign::Right => parley::Alignment::Right,
        crate::font::TextAlign::Center => parley::Alignment::Center,
        crate::font::TextAlign::Justified => parley::Alignment::Justify,
        crate::font::TextAlign::End => parley::Alignment::End,
    };
    layout_editor.set_alignment(alignment);
    let text_layout = layout_editor.layout(&mut font_manager.font_cx, &mut font_manager.layout_cx);

    if !text_layout.is_empty() {
        let mut size = Vec2::new(0.0, 0.0);
        text_layout.lines().for_each(|l| {
            let line_metrics = l.metrics();
            size.x = size.x.max(line_metrics.advance + 1.0);
            size.y += line_metrics.line_height;
        });
        Some(LayoutMeasure::Fixed(super::measure::FixedMeasure { size }))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use bevy::ecs::system::RunSystemOnce;

    use super::*;
    use crate::{
        children::WidgetChildren,
        prelude::{StackingTier, Units, WidgetFlexDirection, WidgetZ},
        svg::SvgAsset,
        WidgetRegisterExt,
    };

    #[derive(Component, Reflect, Default, Clone)]
    struct TestWidget;
    impl crate::context::Widget for TestWidget {}

    fn setup_app() -> App {
        let mut app = App::new();
        app.register_widget::<TestWidget>();
        app.init_resource::<UiLayout>();
        app.init_resource::<FontManager>();
        let default_font = Handle::<bevy_vello::prelude::VelloFont>::default();
        // Registers a placeholder family for the default font handle so `measure_text` can
        // run at all -- see `FontManager::vello_to_family`'s doc comment. No real font asset
        // is loaded, so parley falls back to a system font; the resulting metrics are real
        // (and respond to wrap width), just not any *specific* font's.
        app.world_mut()
            .resource_mut::<FontManager>()
            .vello_to_family
            .insert(default_font.id(), "sans-serif".into());
        app.world_mut().insert_resource(DefaultFont(default_font));
        app.init_resource::<Assets<Image>>();
        app.init_resource::<Assets<SvgAsset>>();
        app
    }

    fn run_layout(app: &mut App) {
        app.world_mut().run_system_once(run).unwrap();
    }

    /// Unlike `run_layout` (`run_system_once`, which re-initializes a fresh `System` -- and
    /// so a fresh `last_run` change tick -- on every single call, making every `Changed<T>`
    /// filter match unconditionally every time), this schedules `run` once and drives it via
    /// real `app.update()` frames, so `Changed<WoodpeckerStyle>` etc. only match entities
    /// actually mutated since the *previous* frame -- the real semantics production code runs
    /// under. Needed for any test asserting that something *stops* being independently dirty
    /// across frames (e.g. a `Units::Calc` entity whose own style never changes again after
    /// its first render) actually gets picked up the way a real, continuously-running app
    /// would.
    fn setup_scheduled_app() -> App {
        let mut app = setup_app();
        app.add_systems(Update, run);
        app
    }

    /// Regression test for the skip-if-clean fast path: mutating a child's
    /// `WoodpeckerStyle` must still update its `WidgetLayout` within the very next
    /// `run()` call, not get silently dropped by the dirty check.
    #[test]
    fn changed_child_style_updates_layout_next_run() {
        let mut app = setup_app();
        let root = app
            .world_mut()
            .spawn((
                TestWidget,
                WoodpeckerStyle {
                    width: 200.0.into(),
                    height: 200.0.into(),
                    ..Default::default()
                },
                WidgetChildren::default(),
            ))
            .id();
        let child = app
            .world_mut()
            .spawn((
                TestWidget,
                WoodpeckerStyle {
                    width: 50.0.into(),
                    height: 50.0.into(),
                    ..Default::default()
                },
                WidgetChildren::default(),
                ChildOf(root),
            ))
            .id();
        app.world_mut()
            .resource_mut::<WoodpeckerContext>()
            .set_root_widget(root);

        run_layout(&mut app);
        let initial_width = app.world().get::<WidgetLayout>(child).unwrap().width();
        assert_eq!(initial_width, 50.0);

        // Idle frame: nothing changed, `WidgetLayout` must be untouched (still correct,
        // just proving the fast path doesn't corrupt anything by skipping).
        run_layout(&mut app);
        assert_eq!(
            app.world().get::<WidgetLayout>(child).unwrap().width(),
            50.0
        );

        // Mutate the child's style and confirm the very next `run()` picks it up --
        // this is what the skip-if-clean dirty check must not defeat.
        app.world_mut()
            .get_mut::<WoodpeckerStyle>(child)
            .unwrap()
            .width = 90.0.into();
        run_layout(&mut app);
        assert_eq!(
            app.world().get::<WidgetLayout>(child).unwrap().width(),
            90.0,
            "a changed child style must still update WidgetLayout on the next run(), even \
             with the skip-if-clean fast path in place"
        );
    }

    /// A `Units::Calc` width is resolved against the parent's own last-committed layout
    /// size before it ever reaches taffy -- see `WoodpeckerStyle::resolve_calc`. Two
    /// `run_layout` calls are needed for the same one-frame-lag reason as
    /// `parent_resize_remeasures_a_stable_text_child_via_gating` above: the child's very
    /// first frame measures against a not-yet-`compute()`-d root (parent size `Vec2::ZERO`),
    /// so only the second frame's `WidgetLayout` reflects the real, committed parent width.
    #[test]
    fn calc_width_resolves_against_the_parents_committed_width() {
        let mut app = setup_app();
        let root = app
            .world_mut()
            .spawn((
                TestWidget,
                WoodpeckerStyle {
                    width: 300.0.into(),
                    height: 200.0.into(),
                    ..Default::default()
                },
                WidgetChildren::default(),
            ))
            .id();
        let child = app
            .world_mut()
            .spawn((
                TestWidget,
                WoodpeckerStyle {
                    width: Units::Calc {
                        percent: 100.0,
                        pixels: -40.0,
                    },
                    height: 50.0.into(),
                    ..Default::default()
                },
                WidgetChildren::default(),
                ChildOf(root),
            ))
            .id();
        app.world_mut()
            .resource_mut::<WoodpeckerContext>()
            .set_root_widget(root);

        run_layout(&mut app);
        run_layout(&mut app);
        assert_eq!(
            app.world().get::<WidgetLayout>(child).unwrap().width(),
            260.0,
            "calc(100% - 40px) against a 300px-wide parent should resolve to 260px"
        );
    }

    /// Regression test: a `Units::Calc` entity's own `WoodpeckerStyle` never changes again
    /// after it's first spawned (the `calc` expression itself is static), so once its very
    /// first render resolves against the root's not-yet-computed size (`Vec2::ZERO` --
    /// `calc(100% - 40px)` collapsing to a nonsensical `-40px`), nothing about *this*
    /// entity's own `Changed<T>` would ever fire again to re-resolve it against the root's
    /// real, since-committed width -- exactly the bug `calc_stale`/`stale_calc` exist to
    /// catch. Uses `setup_scheduled_app`, not `run_layout`'s `run_system_once`: the latter
    /// re-initializes a fresh system (and so a fresh, maximally-old `last_run` tick) on
    /// every call, which makes every `Changed<T>` filter match unconditionally regardless of
    /// whether anything actually changed since the previous call -- silently masking exactly
    /// this bug class.
    #[test]
    fn calc_width_recovers_once_the_parents_real_size_commits_across_real_frames() {
        let mut app = setup_scheduled_app();
        let root = app
            .world_mut()
            .spawn((
                TestWidget,
                WoodpeckerStyle {
                    width: 300.0.into(),
                    height: 200.0.into(),
                    ..Default::default()
                },
                WidgetChildren::default(),
            ))
            .id();
        let child = app
            .world_mut()
            .spawn((
                TestWidget,
                WoodpeckerStyle {
                    width: Units::Calc {
                        percent: 100.0,
                        pixels: -40.0,
                    },
                    height: 50.0.into(),
                    ..Default::default()
                },
                WidgetChildren::default(),
                ChildOf(root),
            ))
            .id();
        app.world_mut()
            .resource_mut::<WoodpeckerContext>()
            .set_root_widget(root);

        // Several real frames, with nothing further ever mutated on `child`'s own
        // `WoodpeckerStyle` -- the exact condition under which the bug this guards against
        // only manifests after the tree has otherwise gone quiet.
        for _ in 0..5 {
            app.update();
        }

        assert_eq!(
            app.world().get::<WidgetLayout>(child).unwrap().width(),
            260.0,
            "calc(100% - 40px) must settle to 260px against the root's real committed \
             300px width, not stay stuck at whatever it resolved to on its own first frame"
        );
    }

    /// A root viewport resize touches no `Changed<T>` the dirty check would otherwise see
    /// (its size is read fresh from the root's own style every frame, not mutated), so
    /// `UiLayout::last_root_size` has to be tracked and compared explicitly.
    #[test]
    fn root_resize_is_detected_without_any_changed_component() {
        let mut app = setup_app();
        let root = app
            .world_mut()
            .spawn((
                TestWidget,
                WoodpeckerStyle {
                    width: 200.0.into(),
                    height: 200.0.into(),
                    ..Default::default()
                },
                WidgetChildren::default(),
            ))
            .id();
        app.world_mut()
            .resource_mut::<WoodpeckerContext>()
            .set_root_widget(root);

        run_layout(&mut app);
        assert_eq!(
            app.world().resource::<UiLayout>().last_root_size,
            Some(Vec2::new(200.0, 200.0))
        );

        // Settle: an idle frame must not touch `last_root_size`.
        run_layout(&mut app);
        assert_eq!(
            app.world().resource::<UiLayout>().last_root_size,
            Some(Vec2::new(200.0, 200.0))
        );

        // Resize without mutating any *other* widget's components -- root style itself
        // does change here (that's the whole point), but nothing on `child` does.
        app.world_mut()
            .get_mut::<WoodpeckerStyle>(root)
            .unwrap()
            .width = 300.0.into();
        run_layout(&mut app);
        assert_eq!(
            app.world().resource::<UiLayout>().last_root_size,
            Some(Vec2::new(300.0, 200.0)),
            "a root viewport resize must be detected even though it's read fresh from \
             styles every frame rather than mutated"
        );
    }

    /// Forces a non-idle `run()` (a `Changed<WoodpeckerStyle>` on `entity`) without touching
    /// anything that affects layout math -- used to flush an already-correct-but-not-yet-
    /// applied measurement through an extra frame, the same one-frame lag `measure_text`
    /// already has against a *just-created* parent's committed (pre-`compute()`) layout.
    fn touch_without_resizing(app: &mut App, entity: Entity) {
        let mut style = app.world_mut().get_mut::<WoodpeckerStyle>(entity).unwrap();
        style.opacity = if style.opacity == 1.0 { 0.999 } else { 1.0 };
    }

    /// Regression test for Phase 9's incremental `set_style` gating: a text node whose own
    /// `WoodpeckerStyle`/`WidgetRender` never change must still re-wrap when its *parent*
    /// resizes, even though that touches no `Changed<T>` on the text node itself -- this is
    /// exactly the "higher-risk" cascade the gating's own doc comment (`measures_via_parent_width`)
    /// flags. Multiple `run_layout` calls are needed because `measure_text` measures against
    /// the parent's last *committed* (post-`compute()`) layout, one frame behind the style
    /// that produced it -- an existing characteristic of this measure pipeline, not something
    /// this test is trying to avoid.
    #[test]
    fn parent_resize_remeasures_a_stable_text_child_via_gating() {
        let mut app = setup_app();
        let root = app
            .world_mut()
            .spawn((
                TestWidget,
                WoodpeckerStyle {
                    width: 400.0.into(),
                    height: 200.0.into(),
                    flex_direction: WidgetFlexDirection::Column,
                    ..Default::default()
                },
                WidgetChildren::default(),
            ))
            .id();
        let text_child = app
            .world_mut()
            .spawn((
                TestWidget,
                WoodpeckerStyle {
                    width: Units::Percentage(100.0),
                    font_size: 16.0,
                    text_wrap: crate::styles::TextWrap::Word,
                    ..Default::default()
                },
                WidgetRender::Text {
                    content: "one two three four five six seven eight nine ten".into(),
                },
                WidgetChildren::default(),
                ChildOf(root),
            ))
            .id();
        app.world_mut()
            .resource_mut::<WoodpeckerContext>()
            .set_root_widget(root);

        // Frame 1: both entities are brand new (dirty regardless of gating). `text_child`'s
        // own measurement here is against a not-yet-`compute()`-d root, so it's not a usable
        // baseline on its own.
        run_layout(&mut app);
        // Frame 2: `root`'s style changes (unrelated to width) -- `text_child` is untouched,
        // so it only gets remeasured this frame via the parent-width-changed path, now against
        // root's real, frame-1-committed 400px width. This is the "wide" baseline.
        touch_without_resizing(&mut app, root);
        run_layout(&mut app);
        let wide_height = app
            .world()
            .get::<WidgetLayout>(text_child)
            .unwrap()
            .height();

        // Frame 3: root actually narrows. `text_child` still untouched.
        app.world_mut()
            .get_mut::<WoodpeckerStyle>(root)
            .unwrap()
            .width = 60.0.into();
        run_layout(&mut app);
        // Frame 4: another unrelated touch flushes the now-committed 60px width through to
        // `text_child`'s measurement, the same way frame 2 did for the 400px width.
        touch_without_resizing(&mut app, root);
        run_layout(&mut app);
        let narrow_height = app
            .world()
            .get::<WidgetLayout>(text_child)
            .unwrap()
            .height();

        assert!(
            narrow_height > wide_height,
            "narrowing the parent from 400px to 60px should force the text onto more lines \
             (wide: {wide_height}, narrow: {narrow_height}) -- if this fails, the incremental \
             gating is skipping `text_child`'s remeasure on a parent resize"
        );
    }

    /// `UiLayout::previous_measure_width` (Phase 9's per-text-entity tracking) must be cleaned
    /// up when the entity is removed, mirroring `entity_to_taffy`'s own cleanup in
    /// `UiLayout::remove_child` -- otherwise it grows unboundedly across the lifetime of an
    /// app with churny text content (e.g. a virtualized list).
    #[test]
    fn removing_a_text_entity_cleans_up_its_tracked_measure_width() {
        let mut app = setup_app();
        let root = app
            .world_mut()
            .spawn((
                TestWidget,
                WoodpeckerStyle {
                    width: 400.0.into(),
                    height: 200.0.into(),
                    ..Default::default()
                },
                WidgetChildren::default(),
            ))
            .id();
        let text_child = app
            .world_mut()
            .spawn((
                TestWidget,
                WoodpeckerStyle {
                    width: Units::Percentage(100.0),
                    ..Default::default()
                },
                WidgetRender::Text {
                    content: "hello".into(),
                },
                WidgetChildren::default(),
                ChildOf(root),
            ))
            .id();
        app.world_mut()
            .resource_mut::<WoodpeckerContext>()
            .set_root_widget(root);

        run_layout(&mut app);
        touch_without_resizing(&mut app, root);
        run_layout(&mut app);
        assert!(
            app.world()
                .resource::<UiLayout>()
                .previous_measure_width
                .contains_key(&text_child),
            "a text entity that's been measured should be tracked"
        );

        app.world_mut().despawn(text_child);
        app.world_mut()
            .resource_mut::<UiLayout>()
            .remove_child(text_child);
        assert!(
            !app.world()
                .resource::<UiLayout>()
                .previous_measure_width
                .contains_key(&text_child),
            "removing the entity should also drop its tracked measure width"
        );
    }

    /// `DatePicker` never sets its own `z_index` -- it composes `Popover` internally and
    /// relies entirely on `Popover`'s own `StackingTier::Popover` tier. This pins the actual
    /// property that matters ("a `Popover`-tier panel always resolves above a `Modal`-tier
    /// panel"), which is what makes a `DatePicker`'s calendar popup render/pick correctly
    /// above an open `Modal` it happens to be nested inside, without `DatePicker` needing any
    /// tier of its own.
    #[test]
    fn popover_tier_resolves_above_modal_tier() {
        let mut app = setup_app();
        let root = app
            .world_mut()
            .spawn((
                TestWidget,
                WoodpeckerStyle {
                    width: 800.0.into(),
                    height: 600.0.into(),
                    ..Default::default()
                },
                WidgetChildren::default(),
            ))
            .id();
        let modal_panel = app
            .world_mut()
            .spawn((
                TestWidget,
                WoodpeckerStyle {
                    z_index: Some(WidgetZ::Global(StackingTier::Modal as u32)),
                    ..Default::default()
                },
                StackingContext,
                WidgetChildren::default(),
                ChildOf(root),
            ))
            .id();
        // Stands in for `DatePicker`'s internal `Popover` floating-content panel -- same
        // z_index a real `DatePicker`'s calendar popup resolves to, since it never overrides
        // `Popover`'s own tier.
        let datepicker_popover_panel = app
            .world_mut()
            .spawn((
                TestWidget,
                WoodpeckerStyle {
                    z_index: Some(WidgetZ::Global(StackingTier::Popover as u32)),
                    ..Default::default()
                },
                StackingContext,
                WidgetChildren::default(),
                ChildOf(root),
            ))
            .id();
        app.world_mut()
            .resource_mut::<WoodpeckerContext>()
            .set_root_widget(root);

        run_layout(&mut app);

        let modal_z = app.world().get::<WidgetLayout>(modal_panel).unwrap().z;
        let popover_z = app
            .world()
            .get::<WidgetLayout>(datepicker_popover_panel)
            .unwrap()
            .z;
        assert!(
            popover_z > modal_z,
            "a DatePicker's popover panel (z={popover_z}) must resolve above an open Modal \
             (z={modal_z}), even though DatePicker itself sets no z_index of its own"
        );
    }

    /// Regression test for `stale_measured_text`: a text child must re-measure against its
    /// parent's settled width even on a frame where *nothing else* is dirty -- previously,
    /// only an unrelated extra touch (see `parent_resize_remeasures_a_stable_text_child_via_gating`'s
    /// `touch_without_resizing` calls) could flush a parent resize through to a text child's
    /// own measurement; a genuinely idle frame after the resize left it wrapped at the old,
    /// stale width forever. This is exactly what happens in practice when a container's final
    /// size only becomes known a couple of frames after both it and its text child are first
    /// created (routine for any multi-level widget composition), with nothing else remaining
    /// dirty afterward to accidentally flush it.
    #[test]
    fn stale_text_measurement_self_corrects_on_an_otherwise_idle_frame() {
        let mut app = setup_app();
        let root = app
            .world_mut()
            .spawn((
                TestWidget,
                WoodpeckerStyle {
                    width: 400.0.into(),
                    height: 200.0.into(),
                    flex_direction: WidgetFlexDirection::Column,
                    ..Default::default()
                },
                WidgetChildren::default(),
            ))
            .id();
        let narrow_parent = app
            .world_mut()
            .spawn((
                TestWidget,
                WoodpeckerStyle {
                    width: 50.0.into(),
                    ..Default::default()
                },
                WidgetChildren::default(),
                ChildOf(root),
            ))
            .id();
        let text_child = app
            .world_mut()
            .spawn((
                TestWidget,
                WoodpeckerStyle {
                    width: Units::Percentage(100.0),
                    font_size: 16.0,
                    text_wrap: crate::styles::TextWrap::Word,
                    ..Default::default()
                },
                WidgetRender::Text {
                    content: "one two three four five six seven eight nine ten".into(),
                },
                WidgetChildren::default(),
                ChildOf(narrow_parent),
            ))
            .id();
        app.world_mut()
            .resource_mut::<WoodpeckerContext>()
            .set_root_widget(root);

        // Frame 1: everything brand new -- `narrow_parent` has no prior committed layout yet,
        // so `text_child` isn't measured this pass (the same short-circuit
        // `parent_resize_remeasures_a_stable_text_child_via_gating` relies on); this frame's
        // own `compute()` commits `narrow_parent`'s 50px width for the first time.
        run_layout(&mut app);
        // Frame 2: `text_child` gets its first real measurement, against the now-committed
        // 50px width -- wraps onto several lines.
        run_layout(&mut app);
        let narrow_height = app
            .world()
            .get::<WidgetLayout>(text_child)
            .unwrap()
            .height();

        // Widen the parent. Nothing about `text_child` itself changes.
        app.world_mut()
            .get_mut::<WoodpeckerStyle>(narrow_parent)
            .unwrap()
            .width = 400.0.into();
        // Frame 3: `narrow_parent`'s own `Changed<WoodpeckerStyle>` forces a walk that commits
        // its new 400px width -- but `text_child`'s own measurement this same pass still reads
        // the *previous* frame's (50px) committed parent layout (the inherent one-frame lag),
        // so it's still wrapped after this frame too.
        run_layout(&mut app);

        // Frame 4: a genuinely idle frame -- nothing touches any `Changed<T>` here at all.
        // Without `stale_measured_text`, `anything_dirty` would be false and this frame would
        // be skipped entirely, leaving `text_child` wrapped at the stale 50px measurement
        // forever (this is the bug: no further interaction ever fixes it on its own).
        run_layout(&mut app);
        let wide_height = app
            .world()
            .get::<WidgetLayout>(text_child)
            .unwrap()
            .height();

        assert!(
            wide_height < narrow_height,
            "a text entity must re-measure against its parent's settled width even on a frame \
             where nothing else is dirty, not stay stuck wrapped at a stale width indefinitely \
             (narrow: {narrow_height}, wide: {wide_height})"
        );
    }

    /// Regression test: a text entity narrower than its parent via `max_width` must wrap (and
    /// commit a height) against that narrower width, not its parent's full width --
    /// previously `measure_text` always fed the *parent's* full width to the text shaper
    /// regardless of `max_width`, so a paragraph capped at (say) 100px inside a 400px-wide
    /// parent measured/wrapped as if it had the full 400px, then got visually clamped to
    /// 100px at paint time -- wrapping onto more lines than the committed (too-short) height
    /// reserved, overlapping whatever followed it.
    #[test]
    fn max_width_narrower_than_parent_is_respected_by_text_wrapping() {
        let mut app = setup_app();
        let root = app
            .world_mut()
            .spawn((
                TestWidget,
                WoodpeckerStyle {
                    width: 400.0.into(),
                    height: 400.0.into(),
                    flex_direction: WidgetFlexDirection::Column,
                    ..Default::default()
                },
                WidgetChildren::default(),
            ))
            .id();
        let unconstrained_text = app
            .world_mut()
            .spawn((
                TestWidget,
                WoodpeckerStyle {
                    width: Units::Percentage(100.0),
                    font_size: 16.0,
                    text_wrap: crate::styles::TextWrap::Word,
                    ..Default::default()
                },
                WidgetRender::Text {
                    content: "one two three four five six seven eight nine ten".into(),
                },
                WidgetChildren::default(),
                ChildOf(root),
            ))
            .id();
        let max_width_text = app
            .world_mut()
            .spawn((
                TestWidget,
                WoodpeckerStyle {
                    width: Units::Percentage(100.0),
                    max_width: 100.0.into(),
                    font_size: 16.0,
                    text_wrap: crate::styles::TextWrap::Word,
                    ..Default::default()
                },
                WidgetRender::Text {
                    content: "one two three four five six seven eight nine ten".into(),
                },
                WidgetChildren::default(),
                ChildOf(root),
            ))
            .id();
        app.world_mut()
            .resource_mut::<WoodpeckerContext>()
            .set_root_widget(root);

        run_layout(&mut app);
        run_layout(&mut app);

        let unconstrained_height = app
            .world()
            .get::<WidgetLayout>(unconstrained_text)
            .unwrap()
            .height();
        let max_width_height = app
            .world()
            .get::<WidgetLayout>(max_width_text)
            .unwrap()
            .height();

        assert!(
            max_width_height > unconstrained_height,
            "a text entity capped by max_width must wrap (and commit a height) against that \
             narrower width, not its parent's full width (unconstrained: \
             {unconstrained_height}, max_width-capped: {max_width_height})"
        );
    }

    /// Acceptance test 1/2 for true CSS-scoped stacking: a top-level `StackingContext` (stands
    /// in for a `.portal()`-ed widget's wrapper, physically relocated to sit directly under
    /// `OverlayRoot`) must still win paint/pick priority over unrelated content nested three
    /// levels deep elsewhere in the tree, even when that nested content's own ancestors are
    /// *also* explicitly z-indexed (just not as high) -- proving the per-level bucket sort at
    /// each nesting level still lets an explicit `Global(n)` escape its own immediate sibling
    /// group correctly, the same way it always could.
    #[test]
    fn escape_preserved_top_level_stacking_context_beats_unrelated_nested_content() {
        let mut app = setup_app();
        let root = app
            .world_mut()
            .spawn((
                TestWidget,
                WoodpeckerStyle {
                    width: 800.0.into(),
                    height: 600.0.into(),
                    ..Default::default()
                },
                WidgetChildren::default(),
            ))
            .id();
        // Stands in for a portaled Modal wrapper -- a real top-level sibling of root's other
        // content, exactly like `OverlayRoot`'s own children are today.
        let portaled_modal = app
            .world_mut()
            .spawn((
                TestWidget,
                WoodpeckerStyle {
                    z_index: Some(WidgetZ::Global(StackingTier::Modal as u32)),
                    ..Default::default()
                },
                StackingContext,
                WidgetChildren::default(),
                ChildOf(root),
            ))
            .id();
        // Unrelated content nested three levels deep, each level with its own modest explicit
        // z_index -- none anywhere near Modal's tier, but real ambient z nonetheless.
        let level1 = app
            .world_mut()
            .spawn((
                TestWidget,
                WoodpeckerStyle {
                    z_index: Some(WidgetZ::Global(10)),
                    ..Default::default()
                },
                WidgetChildren::default(),
                ChildOf(root),
            ))
            .id();
        let level2 = app
            .world_mut()
            .spawn((
                TestWidget,
                WoodpeckerStyle {
                    z_index: Some(WidgetZ::Global(20)),
                    ..Default::default()
                },
                WidgetChildren::default(),
                ChildOf(level1),
            ))
            .id();
        let deeply_nested_sibling = app
            .world_mut()
            .spawn((
                TestWidget,
                WoodpeckerStyle {
                    z_index: Some(WidgetZ::Global(30)),
                    ..Default::default()
                },
                WidgetChildren::default(),
                ChildOf(level2),
            ))
            .id();
        app.world_mut()
            .resource_mut::<WoodpeckerContext>()
            .set_root_widget(root);

        run_layout(&mut app);

        let modal_order = app
            .world()
            .get::<WidgetLayout>(portaled_modal)
            .unwrap()
            .order;
        let nested_order = app
            .world()
            .get::<WidgetLayout>(deeply_nested_sibling)
            .unwrap()
            .order;
        assert!(
            modal_order > nested_order,
            "a top-level Modal-tier StackingContext (order={modal_order}) must still paint/pick \
             above unrelated content nested three levels deep (order={nested_order}), even \
             though every level of that nested content has its own (much smaller) explicit \
             z_index"
        );
    }

    /// Acceptance test 2/2 for true CSS-scoped stacking: a deeply-nested, non-portaled
    /// descendant with a huge explicit `Global(n)` must NOT escape its own parent's sibling
    /// group -- a top-level sibling with a merely-large-but-lower z must still win against it.
    /// This is the actual behavior change Phase 6 introduces: under the old flat/global
    /// `(z, order)` comparison, `deeply_nested_huge_z` (z=999_999) would have beaten
    /// `top_level_sibling` (z=100) outright, regardless of ancestry.
    #[test]
    fn scoping_correct_a_deeply_nested_huge_z_does_not_escape_its_stacking_context() {
        let mut app = setup_app();
        let root = app
            .world_mut()
            .spawn((
                TestWidget,
                WoodpeckerStyle {
                    width: 800.0.into(),
                    height: 600.0.into(),
                    ..Default::default()
                },
                WidgetChildren::default(),
            ))
            .id();
        let top_level_sibling = app
            .world_mut()
            .spawn((
                TestWidget,
                WoodpeckerStyle {
                    z_index: Some(WidgetZ::Global(100)),
                    ..Default::default()
                },
                StackingContext,
                WidgetChildren::default(),
                ChildOf(root),
            ))
            .id();
        // A *separate* stacking context, also a direct child of root, with no explicit
        // z_index of its own (ambient/ordinary) -- its descendant's huge z_index must stay
        // trapped inside it.
        let other_stacking_context = app
            .world_mut()
            .spawn((
                TestWidget,
                WoodpeckerStyle::default(),
                StackingContext,
                WidgetChildren::default(),
                ChildOf(root),
            ))
            .id();
        let level1 = app
            .world_mut()
            .spawn((
                TestWidget,
                WoodpeckerStyle::default(),
                WidgetChildren::default(),
                ChildOf(other_stacking_context),
            ))
            .id();
        let level2 = app
            .world_mut()
            .spawn((
                TestWidget,
                WoodpeckerStyle::default(),
                WidgetChildren::default(),
                ChildOf(level1),
            ))
            .id();
        let deeply_nested_huge_z = app
            .world_mut()
            .spawn((
                TestWidget,
                WoodpeckerStyle {
                    z_index: Some(WidgetZ::Global(999_999)),
                    ..Default::default()
                },
                WidgetChildren::default(),
                ChildOf(level2),
            ))
            .id();
        app.world_mut()
            .resource_mut::<WoodpeckerContext>()
            .set_root_widget(root);

        run_layout(&mut app);

        let sibling_order = app
            .world()
            .get::<WidgetLayout>(top_level_sibling)
            .unwrap()
            .order;
        let nested_order = app
            .world()
            .get::<WidgetLayout>(deeply_nested_huge_z)
            .unwrap()
            .order;
        assert!(
            sibling_order > nested_order,
            "a top-level sibling with z=100 (order={sibling_order}) must still beat a \
             descendant nested three levels inside a *different* stacking context with \
             z=999_999 (order={nested_order}) -- z-index must not escape its nearest \
             StackingContext ancestor's own sibling comparison"
        );
    }
}
