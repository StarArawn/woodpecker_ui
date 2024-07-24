use std::sync::Arc;

use bevy::prelude::*;
use bevy_vello::{
    text::VelloFont,
    vello::{
        self,
        glyph::{skrifa::MetadataProvider, Glyph},
        kurbo::{self, Affine, RoundedRectRadii},
        peniko::{self, Brush},
    },
    VelloScene,
};

use crate::{font::FontManager, prelude::WoodpeckerStyle, DefaultFont};

pub(crate) const VARIATIONS: &[(&str, f32)] = &[];

#[derive(Component, Clone)]
pub enum WidgetRender {
    Quad,
    Text { content: String, word_wrap: bool },
    Custom { render: WidgetRenderCustom },
    Layer,
    Image { image_handle: Handle<Image> },
}

impl WidgetRender {
    pub(crate) fn render(
        &self,
        vello_scene: &mut VelloScene,
        layout: &taffy::Layout,
        parent_layout: &taffy::Layout,
        default_font: &DefaultFont,
        font_assets: &Assets<VelloFont>,
        font_manager: &mut FontManager,
        image_assets: &Assets<Image>,
        widget_style: &WoodpeckerStyle,
    ) -> bool {
        let mut did_layer = false;
        let location_x = layout.location.x;
        let location_y = layout.location.y;
        let size_x = layout.size.width;
        let size_y = layout.size.height;

        match self {
            WidgetRender::Quad => {
                let color = widget_style.background_color.to_srgba();
                let border_color = widget_style.border_color.to_srgba();
                let rect = kurbo::RoundedRect::new(
                    location_x as f64 - layout.border.left as f64,
                    location_y as f64 - layout.border.top as f64,
                    location_x as f64 + (size_x as f64 + layout.border.right as f64),
                    location_y as f64 + (size_y as f64 + layout.border.bottom as f64),
                    RoundedRectRadii::new(
                        widget_style.border_radius.top_left.value_or(0.0) as f64,
                        widget_style.border_radius.top_right.value_or(0.0) as f64,
                        widget_style.border_radius.bottom_right.value_or(0.0) as f64,
                        widget_style.border_radius.bottom_left.value_or(0.0) as f64,
                    ),
                );

                vello_scene.fill(
                    peniko::Fill::NonZero,
                    kurbo::Affine::default(),
                    peniko::Color::rgba(
                        border_color.red as f64,
                        border_color.green as f64,
                        border_color.blue as f64,
                        border_color.alpha as f64,
                    ),
                    None,
                    &rect,
                );

                let rect = kurbo::RoundedRect::new(
                    location_x as f64,
                    location_y as f64,
                    location_x as f64 + size_x as f64,
                    location_y as f64 + size_y as f64,
                    RoundedRectRadii::new(
                        widget_style.border_radius.top_left.value_or(0.0) as f64,
                        widget_style.border_radius.top_right.value_or(0.0) as f64,
                        widget_style.border_radius.bottom_right.value_or(0.0) as f64,
                        widget_style.border_radius.bottom_left.value_or(0.0) as f64,
                    ),
                );
                vello_scene.fill(
                    peniko::Fill::NonZero,
                    kurbo::Affine::default(),
                    peniko::Color::rgba(
                        color.red as f64,
                        color.green as f64,
                        color.blue as f64,
                        color.alpha as f64,
                    ),
                    None,
                    &rect,
                );
            }
            WidgetRender::Text { content, word_wrap } => {
                let font_handle = widget_style.font.as_ref().unwrap_or(&default_font.0);

                let Some(vello_font) = font_assets.get(font_handle) else {
                    return false;
                };

                let Some(buffer) = font_manager.layout(
                    Vec2::new(parent_layout.size.width, parent_layout.size.height),
                    widget_style,
                    font_handle,
                    content,
                    *word_wrap,
                ) else {
                    return false;
                };

                let font_ref = font_manager.get_vello_font(font_handle);

                for run in buffer.layout_runs() {
                    let mut glyphs = vec![];
                    for glyph in run.glyphs.iter() {
                        glyphs.push(Glyph {
                            id: glyph.glyph_id as u32,
                            x: glyph.x,
                            y: glyph.y,
                        });
                    }

                    let transform = vello::kurbo::Affine::translate((
                        layout.location.x as f64,
                        layout.location.y as f64 + run.line_y as f64,
                    ));

                    let axes = font_ref.axes();
                    let var_loc = axes.location(VARIATIONS);
                    let color = widget_style.color.to_srgba();
                    vello_scene
                        .draw_glyphs(&vello_font.font)
                        .font_size(widget_style.font_size)
                        .transform(transform)
                        .normalized_coords(var_loc.coords())
                        .brush(&Brush::Solid(vello::peniko::Color::rgba(
                            color.red as f64,
                            color.green as f64,
                            color.blue as f64,
                            color.alpha as f64,
                        )))
                        .draw(vello::peniko::Fill::NonZero, glyphs.into_iter());
                }
            }
            WidgetRender::Custom { render } => {
                render.render(vello_scene, layout);
            }
            WidgetRender::Layer => {
                let mask_blend = vello::peniko::BlendMode::new(
                    vello::peniko::Mix::Normal,
                    vello::peniko::Compose::SrcOver,
                );
                vello_scene.push_layer(
                    mask_blend,
                    widget_style.opacity,
                    Affine::default(),
                    &kurbo::RoundedRect::new(
                        location_x as f64,
                        location_y as f64,
                        location_x as f64 + size_x as f64,
                        location_y as f64 + size_y as f64,
                        RoundedRectRadii::new(
                            widget_style.border_radius.top_left.value_or(0.0) as f64,
                            widget_style.border_radius.top_right.value_or(0.0) as f64,
                            widget_style.border_radius.bottom_right.value_or(0.0) as f64,
                            widget_style.border_radius.bottom_left.value_or(0.0) as f64,
                        ),
                    ),
                );
                did_layer = true;
            }
            WidgetRender::Image { image_handle } => {
                let Some(image) = image_assets.get(image_handle) else {
                    return false;
                };

                fn fit_image(size_to_fit: Vec2, container_size: Vec2) -> f32 {
                    let multipler = size_to_fit.x * size_to_fit.y;
                    let width_scale = container_size.x / size_to_fit.x;
                    let height_scale = container_size.y / size_to_fit.y;
                    if (width_scale * multipler) < (height_scale * multipler) {
                        width_scale
                    } else {
                        height_scale
                    }
                }

                let transform = vello::kurbo::Affine::translate((
                    layout.location.x as f64,
                    layout.location.y as f64,
                ))
                // TODO: Make scale fit optional via styles.
                .then_scale(fit_image(
                    image.size().as_vec2(),
                    Vec2::new(layout.size.width, layout.size.height),
                ) as f64);

                // TODO: Cache these..
                let vello_image = peniko::Image::new(
                    image.data.clone().into(),
                    peniko::Format::Rgba8,
                    image.size().x,
                    image.size().y,
                );

                vello_scene.draw_image(&vello_image, transform);
            }
        }
        did_layer
    }
}

#[derive(Clone)]
pub struct WidgetRenderCustom {
    inner: Arc<dyn Fn(&mut VelloScene, &taffy::Layout) + Send + Sync>,
}

impl WidgetRenderCustom {
    pub fn new<F>(render: F) -> Self
    where
        F: Fn(&mut VelloScene, &taffy::Layout) + Send + Sync + 'static,
    {
        Self {
            inner: Arc::new(render),
        }
    }

    pub(crate) fn render(&self, vello_scene: &mut VelloScene, layout: &taffy::Layout) {
        self.inner.clone()(vello_scene, layout);
    }
}
