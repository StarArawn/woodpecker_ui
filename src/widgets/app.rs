use crate::{
    children::WidgetChildren,
    prelude::{Units, Widget, WoodpeckerStyle},
    CurrentWidget, WoodpeckerView,
};
use bevy::{
    camera::{CameraProjection, RenderTarget, ScalingMode},
    prelude::*,
    window::PrimaryWindow,
};

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
    camera_query: Query<(&Camera, &Projection, &RenderTarget), With<WoodpeckerView>>,
    images: Res<Assets<Image>>,
) {
    let (_camera, proj, target) = camera_query.single().unwrap();

    let Ok((mut children, mut styles)) = query.get_mut(**entity) else {
        return;
    };

    let camera_size = match target {
        RenderTarget::Window(_) => primary_window.size(),
        RenderTarget::Image(image_render_target) => images
            .get(&image_render_target.handle)
            .unwrap()
            .size()
            .as_vec2(),
        RenderTarget::TextureView(_) => {
            panic!("ManualTextureViewHandle not supported!")
        }
        RenderTarget::None { size } => size.as_vec2(),
    };

    let rect = match proj {
        Projection::Orthographic(orthographic_projection) => {
            let mut proj = orthographic_projection.clone();
            match proj.scaling_mode {
                ScalingMode::WindowSize => Rect {
                    min: Vec2::ZERO,
                    max: camera_size,
                },
                ScalingMode::AutoMin { .. }
                | ScalingMode::AutoMax { .. }
                | ScalingMode::FixedVertical { .. }
                | ScalingMode::FixedHorizontal { .. }
                | ScalingMode::Fixed { .. } => {
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
