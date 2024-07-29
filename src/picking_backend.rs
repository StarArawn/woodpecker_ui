use std::cmp::Ordering;

use bevy::{input::mouse::MouseWheel, prelude::*, window::PrimaryWindow};
use bevy_mod_picking::{
    backend::{HitData, PointerHits},
    events::Pointer,
    focus::HoverMap,
    picking_core::Pickable,
    pointer::{PointerId, PointerLocation},
    prelude::PointerMap,
};

use crate::layout::system::WidgetLayout;

pub(crate) fn system(
    pointers: Query<(&PointerId, &PointerLocation)>,
    cameras: Query<(Entity, &Camera, &GlobalTransform, &OrthographicProjection)>,
    primary_window: Query<Entity, With<PrimaryWindow>>,
    layout_query: Query<(Entity, &WidgetLayout), With<Pickable>>,
    mut output: EventWriter<PointerHits>,
    #[cfg(feature = "debug-render")] mut gizmos: Gizmos,
) {
    let mut sorted_layouts: Vec<_> = layout_query.iter().collect();
    sorted_layouts.sort_by(|a, b| {
        (b.1.order)
            .partial_cmp(&a.1.order)
            .unwrap_or(Ordering::Equal)
    });

    let total = sorted_layouts.len();

    for (pointer, location) in pointers.iter().filter_map(|(pointer, pointer_location)| {
        pointer_location.location().map(|loc| (pointer, loc))
    }) {
        let Some((cam_entity, camera, cam_transform, _cam_ortho)) = cameras
            .iter()
            .filter(|(_, camera, _, _)| camera.is_active)
            .find(|(_, camera, _, _)| {
                camera
                    .target
                    .normalize(Some(match primary_window.get_single() {
                        Ok(w) => w,
                        Err(_) => return false,
                    }))
                    .unwrap()
                    == location.target
            })
        else {
            continue;
        };

        let Some(mut cursor_pos_world) =
            camera.viewport_to_world_2d(cam_transform, location.position)
        else {
            continue;
        };

        let screen_half_size = camera.logical_target_size().unwrap() / 2.0;
        cursor_pos_world.x += screen_half_size.x;
        cursor_pos_world.y = -cursor_pos_world.y + screen_half_size.y;

        let picks = layout_query
            .iter()
            .filter_map(|(entity, layout)| {
                let x = layout.location.x;
                let y = layout.location.y;
                let rect = Rect::new(x, y, x + layout.size.x, y + layout.size.y);
                if rect.contains(cursor_pos_world) {
                    // Draw lines
                    #[cfg(feature = "debug-render")]
                    {
                        let half_size = rect.size() / 2.;
                        fn rect_inner(size: Vec2) -> [Vec2; 4] {
                            let half_size = size / 2.;
                            let tl = Vec2::new(-half_size.x, half_size.y);
                            let tr = Vec2::new(half_size.x, half_size.y);
                            let bl = Vec2::new(-half_size.x, -half_size.y);
                            let br = Vec2::new(half_size.x, -half_size.y);
                            [tl, tr, br, bl]
                        }
                        let [tl, tr, br, bl] = rect_inner(rect.size()).map(|vec2| {
                            let pos = rect.min + half_size + vec2;
                            Vec2::new(pos.x, -pos.y)
                                + Vec2::new(-screen_half_size.x, screen_half_size.y)
                        });
                        gizmos.linestrip_2d([tl, tr, br, bl, tl], Srgba::RED);
                    }

                    Some((
                        entity,
                        // Is 10k entities enough? :shrug:
                        HitData::new(cam_entity, total as f32 - layout.order as f32, None, None),
                    ))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        let order = camera.order as f32;
        output.send(PointerHits::new(*pointer, picks, order));
    }
}

/// When a user scrolls the mouse wheel over an entity.
#[derive(Debug, Default, Reflect, Clone, Copy)]
pub struct MouseWheelScroll {
    pub scroll: Vec2,
}

pub fn mouse_wheel_system(
    // Input
    hover_map: Res<HoverMap>,
    pointer_map: Res<PointerMap>,
    pointers: Query<&PointerLocation>,
    // Bevy Input
    mut evr_scroll: EventReader<MouseWheel>,
    mut pointer_scroll: EventWriter<Pointer<MouseWheelScroll>>,
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
            pointer_scroll.send(Pointer::new(
                pointer_id,
                location.clone(),
                hovered_entity,
                MouseWheelScroll { scroll },
            ));
        }
    }
}
