use bevy::{
    ecs::system::SystemParam,
    input::mouse::MouseWheel,
    picking::{
        backend::{HitData, PointerHits},
        hover::HoverMap,
        pointer::{PointerId, PointerLocation, PointerMap},
    },
    prelude::*,
    window::PrimaryWindow,
};

use crate::{
    context::WoodpeckerContext,
    layout::system::WidgetLayout,
    styles::{WidgetDisplay, WidgetVisibility, WoodpeckerStyle},
    WoodpeckerView,
};

pub(crate) fn system(
    context: Res<WoodpeckerContext>,
    pointers: Query<(&PointerId, &PointerLocation)>,
    cameras: Query<(Entity, &Camera, &GlobalTransform, &Projection), With<WoodpeckerView>>,
    primary_window: Single<(Entity, &Window), With<PrimaryWindow>>,
    layout_query: Query<(&WidgetLayout, &WoodpeckerStyle)>,
    child_query: Query<&Children>,
    pickable_query: Query<&Pickable>,
    mut output: MessageWriter<PointerHits>,
    #[cfg(feature = "debug-render")] mut gizmos: Gizmos,
) {
    for (pointer, location) in pointers.iter().filter_map(|(pointer, pointer_location)| {
        pointer_location.location().map(|loc| (pointer, loc))
    }) {
        let Some((cam_entity, camera, _cam_transform, _cam_ortho)) =
            cameras.iter().find(|(_, camera, _, _)| camera.is_active)
        else {
            continue;
        };

        let (offset, size, scale) = compute_letterboxed_transform(
            primary_window.1.size(),
            camera.logical_target_size().unwrap(),
        );

        let cursor_pos_world =
            ((location.position - offset) / size) * camera.logical_target_size().unwrap();

        // We need to walk the tree here because of visibility. If a parent is hidden it's children shouldn't be hit with clicks.
        let mut candidates = vec![];
        process_entity(
            context.get_root_widget(),
            cursor_pos_world,
            offset / 2.0,
            primary_window.1.size() / 2.0,
            Vec2::splat(scale),
            #[cfg(feature = "debug-render")]
            &mut gizmos,
            &layout_query,
            &child_query,
            &pickable_query,
            &mut candidates,
        );
        // `layout.order` (assigned by `traverse_layout_update`'s bucketed, fully-nested-aware
        // walk -- see `layout::system::order_children_for_paint`) is already a single,
        // globally-comparable, correctly-nested paint-order rank on its own: no separate
        // collect-then-sort re-ranking pass is needed anymore (that used to matter when `z`
        // was also part of the comparison key -- see the removed `rank_hits`). A `u32` cast to
        // f32 stays exact up to ~16 million entities, far beyond any realistic tree.
        let picks = candidates
            .into_iter()
            .map(|(entity, order)| {
                (
                    entity,
                    HitData::new(cam_entity, -(order as f32), None, None),
                )
            })
            .collect();

        let order = camera.order as f32;
        output.write(PointerHits::new(*pointer, picks, order));
    }
}

/// Walks `entity`'s subtree in `Children` order, calling `on_candidate(entity, layout)` for
/// every widget whose rect contains `cursor` -- as long as it's visible (not `Hidden`/
/// `display: None`/near-zero opacity), `excluded` doesn't reject it, and `is_candidate`
/// accepts it. An `excluded` entity (and its entire subtree) is skipped outright -- e.g.
/// devtools' own inspector panel must never be walked while it's scanning the *inspected* app
/// for a click-to-select target. A hidden entity's subtree is also skipped (a hidden parent
/// implies hidden children); an entity that merely fails `is_candidate` is still walked into
/// (its children may still be candidates) -- only `on_candidate` is withheld for it.
///
/// Shared by `process_entity` (real pointer picking, `Pickable`-gated) and
/// `devtools::pick::scan_entity` (click-to-select, every widget with a `WidgetLayout` is a
/// candidate) -- both need the exact same "what's under this point, in what tree order"
/// answer, just filtered differently.
pub(crate) fn walk_hit_candidates(
    entity: Entity,
    cursor: Vec2,
    layout_query: &Query<(&WidgetLayout, &WoodpeckerStyle)>,
    children_query: &Query<&Children>,
    excluded: &impl Fn(Entity) -> bool,
    is_candidate: &impl Fn(Entity) -> bool,
    on_candidate: &mut impl FnMut(Entity, &WidgetLayout),
) {
    if excluded(entity) {
        return;
    }

    if let Ok((layout, style)) = layout_query.get(entity) {
        // Don't even process children if a parent is hidden.
        //
        // `WidgetDisplay::None` must be checked explicitly (not just `WidgetVisibility::Hidden`):
        // a hoisted `position: Fixed` descendant keeps its last real, non-zero `WidgetLayout`
        // even while its logical parent is hidden, so without this check it would stay
        // clickable through (e.g. a closed `WoodpeckerWindow` stealing clicks).
        if matches!(style.visibility, WidgetVisibility::Hidden)
            || matches!(style.display, WidgetDisplay::None)
            || style.opacity < 0.001
        {
            return;
        }

        if is_candidate(entity) {
            let x = layout.location.x;
            let y = layout.location.y;
            let rect = Rect::new(x, y, x + layout.size.x, y + layout.size.y);
            if rect.contains(cursor) {
                on_candidate(entity, layout);
            }
        }
    }

    let Ok(children) = children_query.get(entity) else {
        return;
    };

    for &child in children {
        walk_hit_candidates(
            child,
            cursor,
            layout_query,
            children_query,
            excluded,
            is_candidate,
            on_candidate,
        );
    }
}

fn process_entity(
    entity: Entity,
    cursor_pos_world: Vec2,
    offset: Vec2,
    screen_half_size: Vec2,
    // This is the difference in size between the primary window and the UI camera.
    // It's only used for scalling the debug renderer back up to screenspace.
    scale: Vec2,
    #[cfg(feature = "debug-render")] gizmos: &mut Gizmos,
    layout_query: &Query<(&WidgetLayout, &WoodpeckerStyle)>,
    child_query: &Query<&Children>,
    pickable_query: &Query<&Pickable>,
    candidates: &mut Vec<(Entity, u32)>,
) {
    let _ = (offset, screen_half_size, scale);
    walk_hit_candidates(
        entity,
        cursor_pos_world,
        layout_query,
        child_query,
        &|_entity| false,
        &|entity| pickable_query.contains(entity),
        &mut |entity, layout| {
            #[cfg(feature = "debug-render")]
            {
                let x = layout.location.x;
                let y = layout.location.y;
                let rect = Rect::new(x, y, x + layout.size.x, y + layout.size.y);
                let half_size = offset + rect.size() * scale / 2.;
                fn rect_inner(size: Vec2) -> [Vec2; 4] {
                    let half_size = size / 2.;
                    let tl = Vec2::new(-half_size.x, half_size.y);
                    let tr = Vec2::new(half_size.x, half_size.y);
                    let bl = Vec2::new(-half_size.x, -half_size.y);
                    let br = Vec2::new(half_size.x, -half_size.y);
                    [tl, tr, br, bl]
                }
                let [tl, tr, br, bl] = rect_inner(rect.size() * scale).map(|vec2| {
                    let pos = offset + rect.min * scale + half_size + vec2;
                    Vec2::new(pos.x, -pos.y) + Vec2::new(-screen_half_size.x, screen_half_size.y)
                });
                gizmos.linestrip_2d([tl, tr, br, bl, tl], Srgba::RED);
            }
            candidates.push((entity, layout.order));
        },
    );
}

/// When a user scrolls the mouse wheel over an entity.
#[derive(Debug, Default, Reflect, Clone, Copy)]
pub struct MouseWheelScroll {
    pub scroll: Vec2,
}

pub fn mouse_wheel_system(
    mut commands: Commands,
    // Input
    hover_map: Res<HoverMap>,
    pointer_map: Res<PointerMap>,
    pointers: Query<&PointerLocation>,
    // Bevy Input
    mut evr_scroll: MessageReader<MouseWheel>,
) {
    let pointer_location = |pointer_id: PointerId| {
        pointer_map
            .get_entity(pointer_id)
            .and_then(|entity| pointers.get(entity).ok())
            .and_then(|pointer| pointer.location.clone())
    };

    // Read once up front, not per hovered entity -- `MessageReader::read()` drains its cursor,
    // so reading it inside the loop below would silently see zero events for every pointer
    // after the first.
    let scrolls: Vec<Vec2> = evr_scroll
        .read()
        .map(|mwe| Vec2::new(mwe.x, mwe.y))
        .collect();
    if scrolls.is_empty() {
        return;
    }

    for (pointer_id, hits) in hover_map.iter() {
        // `hover_map` tracks every entity under the pointer, not just the topmost one --
        // e.g. `VirtualList` nested inside a page's own `ScrollBox` means the pointer is
        // simultaneously "over" both. Only the topmost hit (lowest `HitData::depth` -- see
        // `HitData::new` call sites in this file, which set `depth = -(order as f32)`, so a
        // higher paint order sorts to a lower/more-negative depth) should ever receive the
        // wheel event: each hit here becomes its own independent `commands.trigger` call
        // below, so a handler's own `trigger.propagate(false)` (see `VirtualList`'s wheel
        // observer) can't stop a *different*, separately-triggered entity from also reacting
        // to the same physical scroll -- that's a different bug than event bubbling, and
        // needs fixing here, at the source, not in any one handler.
        let Some((&topmost_entity, _)) = hits.iter().min_by(|a, b| {
            a.1.depth
                .partial_cmp(&b.1.depth)
                .unwrap_or(std::cmp::Ordering::Equal)
        }) else {
            continue;
        };

        let Some(location) = pointer_location(*pointer_id) else {
            debug!(
                "Unable to get location for pointer {:?} during pointer over",
                pointer_id
            );
            continue;
        };

        for &scroll in &scrolls {
            commands.trigger(Pointer::new(
                *pointer_id,
                location.clone(),
                MouseWheelScroll { scroll },
                topmost_entity,
            ));
        }
    }
}

/// Computes how to scale and position a virtual resolution (e.g. 320x180)
/// into a real screen (e.g. 1920x1080) with proper letterboxing or pillarboxing.
///
/// Returns:
/// - `offset`: top-left corner of the scaled virtual area in screen space
/// - `size`: size of the scaled virtual area
/// - `scale`: uniform scale factor
pub fn compute_letterboxed_transform(
    screen_resolution: Vec2,
    target_resolution: Vec2,
) -> (Vec2, Vec2, f32) {
    // Compute uniform scale factor to fit whole target into screen
    let scale_x = screen_resolution.x / target_resolution.x;
    let scale_y = screen_resolution.y / target_resolution.y;
    let scale = scale_x.min(scale_y);

    // Scaled size of the virtual content
    let scaled_size = target_resolution * scale;

    // Centered offset (top-left corner)
    let offset = (screen_resolution - scaled_size) / 2.0;

    (offset, scaled_size, scale)
}

/// A `SystemParam` for converting a screen-space pointer position (e.g. the `position` field of
/// `On<Pointer<Drag>>::pointer_location`) into world/virtual-viewport space, collapsing the
/// [`compute_letterboxed_transform`] dance otherwise hand-copied at every drag/click observer
/// that needs it. Mirrors [`crate::cursor::CursorSetter`]'s shape.
#[derive(SystemParam)]
pub struct PointerWorldPosition<'w, 's> {
    window: Single<'w, 's, &'static Window, With<PrimaryWindow>>,
    camera: Query<'w, 's, &'static Camera, With<WoodpeckerView>>,
}

impl PointerWorldPosition<'_, '_> {
    /// Converts `screen_pos` (screen-space, e.g. `trigger.pointer_location.position`) into
    /// world/virtual-viewport space, accounting for letterboxing/pillarboxing. Returns `None`
    /// if there's no active `WoodpeckerView` camera yet, or it has no logical target size yet
    /// (e.g. during startup, before its render target is sized).
    pub fn convert(&self, screen_pos: Vec2) -> Option<Vec2> {
        let target_size = self.target_size()?;
        let (offset, size, _scale) = compute_letterboxed_transform(self.window.size(), target_size);
        Some(((screen_pos - offset) / size) * target_size)
    }

    /// The camera's current logical target (virtual viewport) size, if a `WoodpeckerView`
    /// camera exists and is sized. Useful alongside `convert` for viewport-relative clamping.
    pub fn target_size(&self) -> Option<Vec2> {
        self.camera.iter().next()?.logical_target_size()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::{system::SystemState, world::World};

    fn spawn_widget(
        world: &mut World,
        parent: Option<Entity>,
        z: u32,
        order: u32,
        rect: (f32, f32, f32, f32),
        style: WoodpeckerStyle,
    ) -> Entity {
        let layout = WidgetLayout(crate::layout::system::ReflectedLayout {
            z,
            order,
            location: Vec2::new(rect.0, rect.1),
            size: Vec2::new(rect.2, rect.3),
            ..Default::default()
        });
        let entity = world.spawn((layout, style)).id();
        if let Some(parent) = parent {
            world.entity_mut(entity).insert(ChildOf(parent));
        }
        entity
    }

    /// Behavior-preservation check for the `process_entity`/`devtools::pick::scan_entity`
    /// unification onto `walk_hit_candidates` (Phase 3 of the z-order redesign): a tree with a
    /// `Pickable` widget, a non-`Pickable` decorative widget, and a widget hidden under a
    /// `display: None` ancestor, all overlapping the same point. The `Pickable`-gated walk
    /// (mirrors `picking_backend::process_entity`) must only ever surface the `Pickable`
    /// widget; the ungated walk (mirrors `devtools::pick::scan_entity`) must surface every
    /// visible widget regardless of `Pickable`; both must skip the hidden subtree entirely.
    #[test]
    fn walk_hit_candidates_matches_pickable_gated_and_ungated_semantics() {
        let mut world = World::new();
        let root = spawn_widget(
            &mut world,
            None,
            0,
            0,
            (0.0, 0.0, 100.0, 100.0),
            WoodpeckerStyle::default(),
        );
        let pickable_child = spawn_widget(
            &mut world,
            Some(root),
            0,
            1,
            (0.0, 0.0, 50.0, 50.0),
            WoodpeckerStyle::default(),
        );
        world.entity_mut(pickable_child).insert(Pickable::default());
        let decorative_child = spawn_widget(
            &mut world,
            Some(root),
            0,
            2,
            (0.0, 0.0, 50.0, 50.0),
            WoodpeckerStyle::default(),
        );
        let hidden_parent = spawn_widget(
            &mut world,
            Some(root),
            0,
            3,
            (0.0, 0.0, 50.0, 50.0),
            WoodpeckerStyle {
                display: crate::styles::WidgetDisplay::None,
                ..Default::default()
            },
        );
        let _hidden_child = spawn_widget(
            &mut world,
            Some(hidden_parent),
            0,
            4,
            (0.0, 0.0, 50.0, 50.0),
            WoodpeckerStyle::default(),
        );

        let mut state: SystemState<(Query<(&WidgetLayout, &WoodpeckerStyle)>, Query<&Children>)> =
            SystemState::new(&mut world);
        let Ok((layout_query, children_query)) = state.get(&world) else {
            panic!("SystemState::get failed");
        };

        let cursor = Vec2::new(10.0, 10.0);

        let mut pickable_gated = vec![];
        walk_hit_candidates(
            root,
            cursor,
            &layout_query,
            &children_query,
            &|_| false,
            &|entity| entity == pickable_child,
            &mut |entity, _| pickable_gated.push(entity),
        );
        assert_eq!(
            pickable_gated,
            vec![pickable_child],
            "a Pickable-gated walk (picking_backend::process_entity's semantics) must only \
             surface the Pickable widget, matching picking behavior today"
        );

        let mut ungated = vec![];
        walk_hit_candidates(
            root,
            cursor,
            &layout_query,
            &children_query,
            &|_| false,
            &|_| true,
            &mut |entity, _| ungated.push(entity),
        );
        assert_eq!(
            ungated,
            vec![root, pickable_child, decorative_child],
            "an ungated walk (devtools::pick::scan_entity's semantics) must surface every \
             visible widget under the cursor, including non-Pickable ones, but the hidden \
             subtree must never appear"
        );
    }

    /// The exact scenario that silently collided under the old `depth = -(z + order *
    /// (1.0/64_000.0))` f32 encoding: 50 overlapping siblings all under one high-tier
    /// `StackingContext` (`z=600_000`, `StackingTier::Devtools`'s value -- ULP there was
    /// ~4000x bigger than the old spacing constant, so *every* one of these would have tied).
    /// `layout.order` is a plain `u32` assigned once per entity by `traverse_layout_update`'s
    /// single monotonic counter (see `layout::system::order_children_for_paint`) -- collecting
    /// it via `walk_hit_candidates` and mapping straight to `HitData::depth = -(order as f32)`
    /// (no separate re-ranking pass, unlike the removed `rank_hits`) must still keep every
    /// candidate distinct, with the highest `order` (the real paint-order winner) on top.
    #[test]
    fn many_same_z_siblings_still_get_distinct_depths_via_order_alone() {
        let mut world = World::new();
        let root = spawn_widget(
            &mut world,
            None,
            600_000,
            0,
            (0.0, 0.0, 100.0, 100.0),
            WoodpeckerStyle::default(),
        );
        let entities: Vec<Entity> = (0..50)
            .map(|i| {
                let entity = spawn_widget(
                    &mut world,
                    Some(root),
                    600_000,
                    i + 1,
                    (0.0, 0.0, 100.0, 100.0),
                    WoodpeckerStyle::default(),
                );
                world.entity_mut(entity).insert(Pickable::default());
                entity
            })
            .collect();

        let mut state: SystemState<(Query<(&WidgetLayout, &WoodpeckerStyle)>, Query<&Children>)> =
            SystemState::new(&mut world);
        let Ok((layout_query, children_query)) = state.get(&world) else {
            panic!("SystemState::get failed");
        };

        let mut candidates = vec![];
        walk_hit_candidates(
            root,
            Vec2::new(10.0, 10.0),
            &layout_query,
            &children_query,
            &|_| false,
            &|entity| entity != root,
            &mut |entity, layout| candidates.push((entity, layout.order)),
        );

        let depths: Vec<f32> = candidates
            .iter()
            .map(|&(_, order)| -(order as f32))
            .collect();
        let mut distinct = depths.clone();
        distinct.sort_by(|a, b| a.partial_cmp(b).unwrap());
        distinct.dedup();
        assert_eq!(
            distinct.len(),
            entities.len(),
            "all 50 same-z siblings must get distinct depths -- this is exactly what the old \
             f32 encoding failed to guarantee at this magnitude"
        );

        let winner = candidates
            .iter()
            .max_by_key(|&&(_, order)| order)
            .map(|&(entity, _)| entity)
            .unwrap();
        assert_eq!(
            winner,
            *entities.last().unwrap(),
            "the sibling with the highest `order` (last in paint order) must win"
        );
    }
}
