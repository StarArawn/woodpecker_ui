use crate::{
    children::WidgetChildren,
    prelude::{Units, Widget, WoodpeckerStyle},
    CurrentWidget, WoodpeckerView,
};
use bevy::{prelude::*, render::camera::CameraProjection, window::PrimaryWindow};

/// The Woodpecker UI App component
#[derive(Component, Widget, Reflect, Default, Clone)]
#[widget_systems(update, render)]
#[require(WidgetChildren, WoodpeckerStyle, Name = Name::new("WoodpeckerApp"))]
pub struct WoodpeckerApp;

pub fn update(
    mut prev_size: Local<Vec2>,
    window_query: Query<(Entity, &Window), (Changed<Window>, With<PrimaryWindow>)>,
) -> bool {
    let should_update = window_query.iter().count() > 0;

    if !should_update {
        return false;
    }

    let Some((_, window)) = window_query.iter().next() else {
        return false;
    };

    if window.size() == *prev_size {
        return false;
    }
    *prev_size = window.size();

    true
}

pub fn render(
    entity: Res<CurrentWidget>,
    mut query: Query<(&mut WidgetChildren, &mut WoodpeckerStyle)>,
    primary_window: Single<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &Projection), With<WoodpeckerView>>,
    images: Res<Assets<Image>>,
) {
    let (camera, proj) = camera_query.single().unwrap();

    let Ok((mut children, mut styles)) = query.get_mut(**entity) else {
        return;
    };

    let camera_size = match &camera.target {
        bevy::render::camera::RenderTarget::Window(_) => primary_window.size(),
        bevy::render::camera::RenderTarget::Image(image_render_target) => images
            .get(&image_render_target.handle)
            .unwrap()
            .size()
            .as_vec2(),
        bevy::render::camera::RenderTarget::TextureView(_) => {
            panic!("ManualTextureViewHandle not supported!")
        }
    };

    let rect = match proj {
        Projection::Orthographic(orthographic_projection) => {
            let mut proj = orthographic_projection.clone();
            match proj.scaling_mode {
                bevy::render::camera::ScalingMode::WindowSize => Rect {
                    min: Vec2::ZERO,
                    max: camera_size,
                },
                bevy::render::camera::ScalingMode::AutoMin { .. }
                | bevy::render::camera::ScalingMode::AutoMax { .. }
                | bevy::render::camera::ScalingMode::FixedVertical { .. }
                | bevy::render::camera::ScalingMode::FixedHorizontal { .. }
                | bevy::render::camera::ScalingMode::Fixed { .. } => {
                    proj.update(camera_size.x, camera_size.y);
                    proj.area
                }
            }
        }
        _ => panic!("Perspective projection is Not supported!"),
    };
    *styles = WoodpeckerStyle {
        width: Units::Pixels(rect.size().x),
        height: Units::Pixels(rect.size().y),
        left: rect.min.x.into(),
        top: rect.min.y.into(),
        ..*styles
    };

    children.apply(entity.as_parent());
}
