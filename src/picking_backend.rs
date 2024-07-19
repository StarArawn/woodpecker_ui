use std::cmp::Ordering;

use bevy::{prelude::*, window::PrimaryWindow};
use bevy_mod_picking::{
    backend::{HitData, PointerHits},
    picking_core::Pickable,
    pointer::{PointerId, PointerLocation},
};

use crate::layout::system::WidgetLayout;

pub fn system(
    pointers: Query<(&PointerId, &PointerLocation)>,
    cameras: Query<(Entity, &Camera, &GlobalTransform, &OrthographicProjection)>,
    primary_window: Query<Entity, With<PrimaryWindow>>,
    layout_query: Query<(Entity, &WidgetLayout), With<Pickable>>,
    mut output: EventWriter<PointerHits>,
) {
    let mut sorted_layouts: Vec<_> = layout_query.iter().collect();
    sorted_layouts.sort_by(|a, b| {
        (b.1.order)
            .partial_cmp(&a.1.order)
            .unwrap_or(Ordering::Equal)
    });

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

        let half_size = camera.logical_target_size().unwrap() / 2.0;
        cursor_pos_world.x += half_size.x;
        cursor_pos_world.y = -cursor_pos_world.y + half_size.y;

        let picks = layout_query
            .iter()
            .filter_map(|(entity, layout)| {
                let x = layout.location.x;
                let y = layout.location.y;
                let rect = Rect::new(x, y, x + layout.size.width, y + layout.size.height);
                if rect.contains(cursor_pos_world) {
                    Some((
                        entity,
                        HitData::new(cam_entity, layout.order as f32, None, None),
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
