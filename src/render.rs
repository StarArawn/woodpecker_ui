use std::{
    hash::{DefaultHasher, Hash, Hasher},
    sync::Arc,
};

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
use image::GenericImage;

use crate::{
    font::FontManager,
    image::ImageManager,
    metrics::WidgetMetrics,
    prelude::{WidgetLayout, WoodpeckerStyle},
    svg::{SvgAsset, SvgManager},
    DefaultFont,
};

pub(crate) const VARIATIONS: &[(&str, f32)] = &[];

/// Used to tell Woodpecker UI's rendering system(vello) how
/// to render a specific widget entity.
#[derive(Component, Clone, Reflect, Default)]
pub enum WidgetRender {
    #[default]
    /// A basic quad shape. Can include borders.
    Quad,
    /// A text shape renderer
    Text {
        /// The text to render
        content: String,
        /// Should the text word wrap
        // TODO: Move to styles..
        word_wrap: bool,
    },
    /// A custom vello renderer.
    /// TODO: Untested, write an example?
    Custom {
        /// A custom widget render function
        #[reflect(ignore)]
        render: WidgetRenderCustom,
    },
    /// A render layer
    ///
    /// Render layers are two things
    /// 1. They clip child content that overflows outside of their own bounds(shape).
    /// 2. They stick children into a new opacity layer. This allows the children to have opacity
    /// as a group instead of individually.
    /// TODO: Allow users to define custom clip shapes (supported by vello we just need to expose somehow)
    Layer,
    /// A simple image renderer
    Image {
        /// A handle to a bevy image.
        handle: Handle<Image>,
    },
    /// A nine patch image
    NinePatch {
        /// An asset handle to a nine patch image.
        handle: Handle<Image>,
        /// A bevy image scale mode.
        scale_mode: ImageScaleMode,
    },
    /// A SVG asset.
    Svg {
        /// A handle to the SVG asset.
        handle: Handle<SvgAsset>,
        /// An optional color that replaces paths and fills within the svg.
        color: Option<Color>,
    },
}

impl WidgetRender {
    /// Sets the color of SVGs and other WidgetRender's that accept colors.
    pub fn set_color(&mut self, color: Color) {
        match self {
            WidgetRender::Quad => {}
            WidgetRender::Text { .. } => {}
            WidgetRender::Custom { .. } => {}
            WidgetRender::Layer => {}
            WidgetRender::Image { .. } => {}
            WidgetRender::NinePatch { .. } => {}
            WidgetRender::Svg {
                color: path_color, ..
            } => {
                *path_color = Some(color);
            }
        }
    }

    pub(crate) fn render(
        &self,
        vello_scene: &mut VelloScene,
        layout: &WidgetLayout,
        parent_layout: &WidgetLayout,
        default_font: &DefaultFont,
        font_assets: &Assets<VelloFont>,
        image_assets: &Assets<Image>,
        svg_assets: &Assets<SvgAsset>,
        font_manager: &mut FontManager,
        svg_manager: &mut SvgManager,
        image_manager: &mut ImageManager,
        metrics: &mut WidgetMetrics,
        widget_style: &WoodpeckerStyle,
    ) -> bool {
        let mut did_layer = false;
        let location_x = layout.location.x;
        let location_y = layout.location.y;
        let size_x = layout.size.x;
        let size_y = layout.size.y;

        if matches!(widget_style.display, crate::styles::WidgetDisplay::None) {
            return false;
        }

        match self {
            WidgetRender::Quad => {
                let color = widget_style.background_color.to_srgba();
                let border_color = widget_style.border_color.to_srgba();
                let rect = kurbo::RoundedRect::new(
                    location_x as f64 - layout.border.left.value_or(0.0) as f64,
                    location_y as f64 - layout.border.top.value_or(0.0) as f64,
                    location_x as f64 + (size_x as f64 + layout.border.right.value_or(0.0) as f64),
                    location_y as f64 + (size_y as f64 + layout.border.bottom.value_or(0.0) as f64),
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
                metrics.increase_quad_counts();
            }
            WidgetRender::Text { content, word_wrap } => {
                let font_handle = widget_style
                    .font
                    .as_ref()
                    .map(|a| Handle::<VelloFont>::Weak(*a))
                    .unwrap_or(default_font.0.clone());

                let Some(vello_font) = font_assets.get(&font_handle) else {
                    return false;
                };

                let Some(buffer) = font_manager.layout(
                    Vec2::new(parent_layout.size.x, parent_layout.size.y),
                    widget_style,
                    &font_handle,
                    content,
                    *word_wrap,
                ) else {
                    return false;
                };

                let font_ref = font_manager.get_vello_font(&font_handle);

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
                render.render(vello_scene, layout, widget_style);
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
            WidgetRender::Image {
                handle: image_handle,
            } => {
                let Some(image) = image_assets.get(image_handle) else {
                    return false;
                };

                let scale = fit_image(
                    image.size().as_vec2(),
                    Vec2::new(layout.size.x, layout.size.y),
                ) as f64;

                let transform = vello::kurbo::Affine::scale(scale).with_translation(
                    bevy_vello::prelude::kurbo::Vec2::new(
                        layout.location.x as f64,
                        layout.location.y as f64,
                    ),
                );

                let vello_image = image_manager
                    .images
                    .entry(image_handle.into())
                    .or_insert_with(|| {
                        peniko::Image::new(
                            image.data.clone().into(),
                            peniko::Format::Rgba8,
                            image.size().x,
                            image.size().y,
                        )
                    });

                vello_scene.draw_image(vello_image, transform);
            }
            WidgetRender::Svg {
                handle,
                color: path_color,
            } => {
                let Some(svg_asset) = svg_assets.get(handle) else {
                    return false;
                };

                let (width, height) = (svg_asset.width, svg_asset.height);

                let transform = vello::kurbo::Affine::scale(fit_image(
                    Vec2::new(width, height),
                    Vec2::new(layout.size.x, layout.size.y),
                ) as f64)
                .with_translation(bevy_vello::prelude::kurbo::Vec2::new(
                    layout.location.x as f64,
                    layout.location.y as f64,
                ));

                let Some(svg_scene) = svg_manager.get_cached(handle, svg_assets, *path_color)
                else {
                    return false;
                };

                vello_scene.append(&svg_scene, Some(transform));
            }
            WidgetRender::NinePatch { handle, scale_mode } => {
                let Some(image) = image_assets.get(handle) else {
                    return false;
                };

                let image_rect = Rect {
                    min: Vec2::ZERO,
                    max: Vec2::new(image.size().x as f32, image.size().y as f32),
                };
                let layout_size = Vec2::new(layout.size.x, layout.size.y);
                let slices = match scale_mode {
                    ImageScaleMode::Sliced(slicer) => {
                        slicer.compute_slices(image_rect, Some(layout_size))
                    }
                    ImageScaleMode::Tiled {
                        tile_x,
                        tile_y,
                        stretch_value,
                    } => {
                        let slice = TextureSlice {
                            texture_rect: image_rect,
                            draw_size: layout_size,
                            offset: Vec2::ZERO,
                        };
                        slice.tiled(*stretch_value, (*tile_x, *tile_y))
                    }
                };

                fn subsection_image_data(image: &mut image::DynamicImage, region: Rect) -> Vec<u8> {
                    let sub_image = image
                        .sub_image(
                            region.min.x as u32,
                            region.min.y as u32,
                            region.size().x as u32,
                            region.size().y as u32,
                        )
                        .to_image();
                    // let _ = sub_image.save_with_format(format!("image{}{}.png", region.min.x, region.min.y), image::ImageFormat::Png);
                    sub_image.as_raw().clone()
                }

                for slice in slices.iter() {
                    let texture_rect_floor = Rect {
                        min: slice.texture_rect.min,
                        max: slice.texture_rect.max,
                    };
                    let min = texture_rect_floor.min.as_uvec2();
                    let max = texture_rect_floor.max.as_uvec2();

                    let mut hasher = DefaultHasher::default();
                    min.hash(&mut hasher);
                    max.hash(&mut hasher);
                    let key = hasher.finish();

                    if !image_manager.nine_patch_slices.contains_key(&key) {
                        let image = image::RgbaImage::from_raw(
                            image.size().x,
                            image.size().y,
                            image.data.clone(),
                        )
                        .unwrap();
                        let mut image: image::DynamicImage = image::DynamicImage::ImageRgba8(image);
                        let sub_section_data =
                            subsection_image_data(&mut image, texture_rect_floor);
                        let vello_image = peniko::Image::new(
                            sub_section_data.into(),
                            peniko::Format::Rgba8,
                            texture_rect_floor.size().x as u32,
                            texture_rect_floor.size().y as u32,
                        );
                        image_manager.nine_patch_slices.insert(key, vello_image);
                    }

                    let vello_image = image_manager.nine_patch_slices.get(&key).unwrap();
                    let scale = slice.draw_size / texture_rect_floor.size();
                    let pos = (
                        slice.offset.x.round() + (layout.size.x / 2.0),
                        -slice.offset.y.round() + (layout.size.y / 2.0),
                    );

                    let transform =
                        vello::kurbo::Affine::scale_non_uniform(scale.x as f64, scale.y as f64)
                            .with_translation(bevy_vello::prelude::kurbo::Vec2::new(
                                (layout.location.x as f64 + pos.0 as f64)
                                    - (slice.draw_size.x as f64 / 2.0),
                                (layout.location.y as f64 + pos.1 as f64)
                                    - (slice.draw_size.y as f64 / 2.0),
                            ));

                    vello_scene.draw_image(vello_image, transform);
                }
            }
        }
        did_layer
    }
}

pub(crate) fn fit_image(size_to_fit: Vec2, container_size: Vec2) -> f32 {
    let multipler = size_to_fit.x * size_to_fit.y;
    let width_scale = container_size.x / size_to_fit.x;
    let height_scale = container_size.y / size_to_fit.y;
    if (width_scale * multipler) < (height_scale * multipler) {
        width_scale
    } else {
        height_scale
    }
}

/// A custom widget vello renderer.
#[derive(Clone)]
pub struct WidgetRenderCustom {
    inner: Arc<dyn Fn(&mut VelloScene, &WidgetLayout, &WoodpeckerStyle) + Send + Sync>,
}

impl Default for WidgetRenderCustom {
    fn default() -> Self {
        Self::new(|_, _, _| {})
    }
}

impl WidgetRenderCustom {
    /// Create a new custom widget render.
    pub fn new<F>(render: F) -> Self
    where
        F: Fn(&mut VelloScene, &WidgetLayout, &WoodpeckerStyle) + Send + Sync + 'static,
    {
        Self {
            inner: Arc::new(render),
        }
    }

    pub(crate) fn render(
        &self,
        vello_scene: &mut VelloScene,
        layout: &WidgetLayout,
        styles: &WoodpeckerStyle,
    ) {
        self.inner.clone()(vello_scene, layout, styles);
    }
}
