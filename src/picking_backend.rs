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

use crate::{
    context::WoodpeckerContext,
    layout::system::WidgetLayout,
    styles::{WidgetVisibility, WoodpeckerStyle},
};

pub(crate) fn system(
    context: Res<WoodpeckerContext>,
    pointers: Query<(&PointerId, &PointerLocation)>,
    cameras: Query<(Entity, &Camera, &GlobalTransform, &Projection)>,
    primary_window: Query<Entity, With<PrimaryWindow>>,
    layout_query: Query<(&WidgetLayout, &WoodpeckerStyle)>,
    child_query: Query<&Children>,
    pickable_query: Query<&Pickable>,
    mut output: EventWriter<PointerHits>,
    #[cfg(feature = "debug-render")] mut gizmos: Gizmos,
) {
    let total = pickable_query.iter().count();

    for (pointer, location) in pointers.iter().filter_map(|(pointer, pointer_location)| {
        pointer_location.location().map(|loc| (pointer, loc))
    }) {
        let Some((cam_entity, camera, cam_transform, _cam_ortho)) = cameras
            .iter()
            .filter(|(_, camera, _, _)| camera.is_active)
            .find(|(_, camera, _, _)| {
                camera
                    .target
                    .normalize(Some(match primary_window.single() {
                        Ok(w) => w,
                        Err(_) => return false,
                    }))
                    .unwrap()
                    == location.target
            })
        else {
            continue;
        };

        let Ok(mut cursor_pos_world) =
            camera.viewport_to_world_2d(cam_transform, location.position)
        else {
            continue;
        };

        let screen_half_size = camera.logical_target_size().unwrap() / 2.0;
        cursor_pos_world.x += screen_half_size.x;
        cursor_pos_world.y = -cursor_pos_world.y + screen_half_size.y;

        // We need to walk the tree here because of visibility. If a parent is hidden it's children shouldn't be hit with clicks.
        let mut picks = vec![];
        process_entity(
            context.get_root_widget(),
            cam_entity,
            cursor_pos_world,
            screen_half_size,
            #[cfg(feature = "debug-render")]
            &mut gizmos,
            &layout_query,
            &child_query,
            &pickable_query,
            &mut picks,
            total,
        );

        // let picks = sorted_layouts
        //     .iter()
        //     .filter_map(|(entity, layout, style)| {
        //         if matches!(style.visibility, WidgetVisibility::Hidden) {
        //             return None;
        //         }
        //         let x = layout.location.x;
        //         let y = layout.location.y;
        //         let rect = Rect::new(x, y, x + layout.size.x, y + layout.size.y);
        //         if rect.contains(cursor_pos_world) {
        //             // Draw lines
        //             #[cfg(feature = "debug-render")]
        //             {
        //                 let half_size = rect.size() / 2.;
        //                 fn rect_inner(size: Vec2) -> [Vec2; 4] {
        //                     let half_size = size / 2.;
        //                     let tl = Vec2::new(-half_size.x, half_size.y);
        //                     let tr = Vec2::new(half_size.x, half_size.y);
        //                     let bl = Vec2::new(-half_size.x, -half_size.y);
        //                     let br = Vec2::new(half_size.x, -half_size.y);
        //                     [tl, tr, br, bl]
        //                 }
        //                 let [tl, tr, br, bl] = rect_inner(rect.size()).map(|vec2| {
        //                     let pos = rect.min + half_size + vec2;
        //                     Vec2::new(pos.x, -pos.y)
        //                         + Vec2::new(-screen_half_size.x, screen_half_size.y)
        //                 });
        //                 gizmos.linestrip_2d([tl, tr, br, bl, tl], Srgba::RED);
        //             }
        //             Some((
        //                 *entity,
        //                 // Is 10k entities enough? :shrug:
        //                 HitData::new(cam_entity, total as f32 - layout.order as f32, None, None),
        //             ))
        //         } else {
        //             None
        //         }
        //     })
        //     .collect::<Vec<_>>();

        let order = camera.order as f32;
        output.write(PointerHits::new(*pointer, picks, order));
    }
}

fn process_entity(
    entity: Entity,
    cam_entity: Entity,
    cursor_pos_world: Vec2,
    screen_half_size: Vec2,
    #[cfg(feature = "debug-render")] gizmos: &mut Gizmos,
    layout_query: &Query<(&WidgetLayout, &WoodpeckerStyle)>,
    child_query: &Query<&Children>,
    pickable_query: &Query<&Pickable>,
    pick_list: &mut Vec<(Entity, HitData)>,
    total: usize,
) {
    if let Ok((layout, style)) = layout_query.get(entity) {
        // Don't even process children if a parent is hidden.
        if matches!(style.visibility, WidgetVisibility::Hidden) || style.opacity < 0.001 {
            return;
        }

        if pickable_query.contains(entity) {
            let x = layout.location.x;
            let y = layout.location.y;
            let rect = Rect::new(x, y, x + layout.size.x, y + layout.size.y);
            if rect.contains(cursor_pos_world) {
                // Draw lines
                let _ = screen_half_size;
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
                pick_list.push((
                    entity,
                    HitData::new(cam_entity, total as f32 - layout.order as f32, None, None),
                ));
            }
        }
    }

    // Process children
    let Ok(children) = child_query.get(entity) else {
        return;
    };

    for child in children {
        process_entity(
            *child,
            cam_entity,
            cursor_pos_world,
            screen_half_size,
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
