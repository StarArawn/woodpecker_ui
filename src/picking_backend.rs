use crate::{
    context::WoodpeckerContext,
    layout::system::WidgetLayout,
    render::WidgetRender,
    styles::{WidgetVisibility, WoodpeckerStyle},
    WoodpeckerView,
};
use bevy::{
    input::mouse::MouseWheel,
    picking::{
        backend::{HitData, PointerHits},
        hover::HoverMap,
        pointer::{PointerId, PointerLocation, PointerMap},
    },
    prelude::*,
    window::PrimaryWindow,
};

pub(crate) fn system(
    context: Res<WoodpeckerContext>,
    pointers: Query<(&PointerId, &PointerLocation)>,
    cameras: Query<(Entity, &Camera, &GlobalTransform, &Projection), With<WoodpeckerView>>,
    primary_window: Single<(Entity, &Window), With<PrimaryWindow>>,
    layout_query: Query<(&WidgetLayout, &WoodpeckerStyle, &WidgetRender)>,
    child_query: Query<&Children>,
    pickable_query: Query<&Pickable>,
    mut output: EventWriter<PointerHits>,
    #[cfg(feature = "debug-render")] mut gizmos: Gizmos,
) {
    let total = pickable_query.iter().count();

    for (pointer, location) in pointers.iter().filter_map(|(pointer, pointer_location)| {
        pointer_location.location().map(|loc| (pointer, loc))
    }) {
        let Some((cam_entity, camera, _cam_transform, _cam_ortho)) = cameras
            .iter()
            .filter(|(_, camera, _, _)| camera.is_active)
            .next()
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
        let mut picks = vec![];
        process_entity(
            None,
            context.get_root_widget(),
            cam_entity,
            cursor_pos_world,
            offset / 2.0,
            primary_window.1.size() / 2.0,
            Vec2::splat(scale),
            #[cfg(feature = "debug-render")]
            &mut gizmos,
            &layout_query,
            &child_query,
            &pickable_query,
            &mut picks,
            total,
        );

        let order = camera.order as f32;
        output.write(PointerHits::new(*pointer, picks, order));
    }
}

fn process_entity(
    mut last_clip: Option<Entity>,
    entity: Entity,
    cam_entity: Entity,
    cursor_pos_world: Vec2,
    offset: Vec2,
    screen_half_size: Vec2,
    // This is the difference in size between the primary window and the UI camera.
    // It's only used for scalling the debug renderer back up to screenspace.
    scale: Vec2,
    #[cfg(feature = "debug-render")] gizmos: &mut Gizmos,
    layout_query: &Query<(&WidgetLayout, &WoodpeckerStyle, &WidgetRender)>,
    child_query: &Query<&Children>,
    pickable_query: &Query<&Pickable>,
    pick_list: &mut Vec<(Entity, HitData)>,
    total: usize,
) {
    if let Ok((layout, style, render)) = layout_query.get(entity) {
        // Don't even process children if a parent is hidden.
        if matches!(style.visibility, WidgetVisibility::Hidden) || style.opacity < 0.001 {
            return;
        }

        let parent_data = last_clip.map(|e| layout_query.get(e).ok()).flatten();

        if matches!(render, WidgetRender::Layer) {
            last_clip = Some(entity);
        }

        if pickable_query.contains(entity) {
            let x = layout.location.x;
            let y = layout.location.y;
            let rect = Rect::new(x, y, x + layout.size.x, y + layout.size.y);
            if rect.contains(cursor_pos_world) {
                // Check if we are in the bounds of the last clip.
                if let Some((layout, _, _)) = parent_data {
                    let x = layout.location.x;
                    let y = layout.location.y;
                    let rect = Rect::new(x, y, x + layout.size.x, y + layout.size.y);
                    if !rect.contains(cursor_pos_world) {
                        return;
                    }
                }

                // Draw lines
                let _ = screen_half_size;
                #[cfg(feature = "debug-render")]
                {
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
                        Vec2::new(pos.x, -pos.y)
                            + Vec2::new(-screen_half_size.x, screen_half_size.y)
                    });
                    gizmos.linestrip_2d([tl, tr, br, bl, tl], Srgba::RED);
                }
                const ORDER_SPACING: f32 = 1.0 / 64_000.0;
                let depth = -(layout.z as f32 + (layout.order as f32 * ORDER_SPACING));
                pick_list.push((entity, HitData::new(cam_entity, depth, None, None)));
            }
        }
    }

    // Process children
    let Ok(children) = child_query.get(entity) else {
        return;
    };

    for child in children {
        process_entity(
            last_clip,
            *child,
            cam_entity,
            cursor_pos_world,
            offset,
            screen_half_size,
            scale,
            #[cfg(feature = "debug-render")]
            gizmos,
            layout_query,
            child_query,
            pickable_query,
            pick_list,
            total,
        );
    }
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
    mut evr_scroll: EventReader<MouseWheel>,
) {
    let pointer_location = |pointer_id: PointerId| {
        pointer_map
            .get_entity(pointer_id)
            .and_then(|entity| pointers.get(entity).ok())
            .and_then(|pointer| pointer.location.clone())
    };

    for (pointer_id, hovered_entity, _hit) in hover_map
        .iter()
        .flat_map(|(id, hashmap)| hashmap.iter().map(|data| (*id, *data.0, data.1.clone())))
    {
        let Some(location) = pointer_location(pointer_id) else {
            debug!(
                "Unable to get location for pointer {:?} during pointer over",
                pointer_id
            );
            continue;
        };

        for mwe in evr_scroll.read() {
            let scroll = Vec2::new(mwe.x, mwe.y);
            commands.trigger_targets(
                Pointer::new(
                    pointer_id,
                    location.clone(),
                    hovered_entity,
                    MouseWheelScroll { scroll },
                ),
                hovered_entity,
            );
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
