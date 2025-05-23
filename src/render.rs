use std::{
    hash::{DefaultHasher, Hash, Hasher},
    sync::Arc,
};

use bevy::{asset::RenderAssetUsages, prelude::*};
use bevy_vello::{
    prelude::VelloFont,
    vello::{
        self,
        kurbo::{self, Affine, RoundedRectRadii},
        peniko::{self, Brush},
        wgpu::{TextureFormat, TextureUsages},
    },
    VelloScene,
};
use image::GenericImage;
use parley::StyleSet;

use crate::{
    convert_render_target::RenderTargetImages,
    font::FontManager,
    image::ImageManager,
    metrics::WidgetMetrics,
    prelude::{RichText, WidgetLayout, WoodpeckerStyle},
    svg::{SvgAsset, SvgManager},
    DefaultFont,
};

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
    },
    /// A rich text shape renderer
    RichText {
        /// The rich text to render
        content: RichText,
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
    ///    as a group instead of individually.
    // TODO: Allow users to define custom clip shapes (supported by vello we just need to expose somehow)
    Layer,
    /// A simple image renderer
    Image {
        /// A handle to a bevy image.
        handle: Handle<Image>,
    },
    /// A bevy render target
    /// Only some texture formats are supported as we convert to rgba8unorm
    RenderTarget {
        /// A bevy render taret.
        handle: Handle<Image>,
    },
    /// A nine patch image
    NinePatch {
        /// An asset handle to a nine patch image.
        handle: Handle<Image>,
        /// A bevy image scale mode.
        scale_mode: SpriteImageMode,
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
            WidgetRender::RichText { .. } => {}
            WidgetRender::Custom { .. } => {}
            WidgetRender::Layer => {}
            WidgetRender::Image { .. } => {}
            WidgetRender::NinePatch { .. } => {}
            WidgetRender::RenderTarget { .. } => {}
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
        _font_assets: &Assets<VelloFont>,
        image_assets: &mut Assets<Image>,
        svg_assets: &Assets<SvgAsset>,
        font_manager: &mut FontManager,
        svg_manager: &mut SvgManager,
        image_manager: &mut ImageManager,
        render_targets: &mut RenderTargetImages,
        metrics: &mut WidgetMetrics,
        widget_style: &WoodpeckerStyle,
        camera_scale: Vec2,
        camera_size: Vec2,
    ) -> bool {
        let mut did_layer = false;
        let location_x = layout.location.x * camera_scale.x;
        let location_y = layout.location.y * camera_scale.y;
        let size_x = layout.size.x * camera_scale.x;
        let size_y = layout.size.y * camera_scale.y;

        if matches!(widget_style.display, crate::styles::WidgetDisplay::None) {
            return false;
        }

        // Screen clipping
        if location_y + size_y < 0.0
            || location_x + size_x < 0.0
            || location_x > camera_size.x
            || location_y > camera_size.y
        {
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
                    peniko::Color::new([
                        border_color.red,
                        border_color.green,
                        border_color.blue,
                        border_color.alpha,
                    ]),
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
                    peniko::Color::new([color.red, color.green, color.blue, color.alpha]),
                    None,
                    &rect,
                );
                metrics.increase_quad_counts();
            }
            WidgetRender::RichText { content } => {
                let font_name = font_manager
                    .get_family(widget_style.font.as_ref().unwrap_or(&default_font.0.id()))
                    .into();
                let mut builder = font_manager.layout_cx.ranged_builder(
                    &mut font_manager.font_cx,
                    &content.text,
                    1.0,
                    true,
                );

                let mut styles = StyleSet::new(widget_style.font_size);
                let color = widget_style.color.to_srgba();
                styles.insert(parley::StyleProperty::Brush(Brush::Solid(
                    peniko::color::AlphaColor::new([
                        color.red,
                        color.green,
                        color.blue,
                        color.alpha,
                    ]),
                )));
                styles.insert(parley::StyleProperty::LineHeight(
                    widget_style
                        .line_height
                        .map(|lh| widget_style.font_size / lh)
                        .unwrap_or(1.2),
                ));
                styles.insert(parley::StyleProperty::FontStack(parley::FontStack::Single(
                    parley::FontFamily::Named(font_name),
                )));
                styles.insert(parley::StyleProperty::OverflowWrap(
                    match widget_style.text_wrap {
                        crate::styles::TextWrap::None => parley::OverflowWrap::Normal,
                        crate::styles::TextWrap::Glyph => parley::OverflowWrap::Anywhere,
                        crate::styles::TextWrap::Word => parley::OverflowWrap::BreakWord,
                        crate::styles::TextWrap::WordOrGlyph => parley::OverflowWrap::Anywhere,
                    },
                ));
                for prop in styles.inner().values() {
                    builder.push_default(prop.to_owned());
                }

                let alignment = match widget_style
                    .text_alignment
                    .unwrap_or(crate::font::TextAlign::Left)
                {
                    crate::font::TextAlign::Left => parley::Alignment::Left,
                    crate::font::TextAlign::Right => parley::Alignment::Right,
                    crate::font::TextAlign::Center => parley::Alignment::Middle,
                    crate::font::TextAlign::Justified => parley::Alignment::Justified,
                    crate::font::TextAlign::End => parley::Alignment::End,
                };

                for color_text in content.highlighted.color_text.iter() {
                    let color = color_text.color.to_srgba();
                    builder.push(
                        parley::StyleProperty::Brush(Brush::Solid(peniko::color::AlphaColor::new(
                            [color.red, color.green, color.blue, color.alpha],
                        ))),
                        color_text.range.clone(),
                    );
                }

                let mut layout = builder.build(&content.text);
                layout.break_all_lines(Some(parent_layout.size.x * camera_scale.x));
                layout.align(
                    Some(parent_layout.size.x * camera_scale.x),
                    alignment,
                    parley::AlignmentOptions::default(),
                );

                for line in layout.lines() {
                    for item in line.items() {
                        let parley::PositionedLayoutItem::GlyphRun(glyph_run) = item else {
                            continue;
                        };

                        let mut x = glyph_run.offset();
                        let y = glyph_run.baseline();
                        let run = glyph_run.run();
                        let font = run.font();
                        let font_size = run.font_size();
                        let synthesis = run.synthesis();
                        let style = glyph_run.style();

                        let posx = location_x;
                        let posy = location_y;

                        // Culling
                        let mut glyph_xform = synthesis
                            .skew()
                            .map(|angle| Affine::skew(angle.to_radians().tan() as f64, 0.0))
                            .unwrap_or_else(vello::kurbo::Affine::default);

                        let trans = glyph_xform.translation();
                        glyph_xform = glyph_xform.with_translation(
                            trans + bevy_vello::vello::kurbo::Vec2::new(posx as f64, posy as f64),
                        );

                        vello_scene
                            .draw_glyphs(font)
                            .hint(true)
                            .font_size(font_size * camera_scale.x)
                            .transform(glyph_xform)
                            .normalized_coords(run.normalized_coords())
                            .brush(&style.brush)
                            .draw(
                                vello::peniko::Fill::NonZero,
                                glyph_run.glyphs().map(|glyph| {
                                    let gx = x + glyph.x;
                                    let gy = y - glyph.y;
                                    x += glyph.advance;
                                    vello::Glyph {
                                        id: glyph.id as _,
                                        x: gx,
                                        y: gy,
                                    }
                                }),
                            );
                    }
                }
            }
            WidgetRender::Text { content } => {
                // TODO: Cache this.
                let mut layout_editor = parley::PlainEditor::new(widget_style.font_size);
                layout_editor.set_text(content);
                let styles = layout_editor.edit_styles();
                styles.insert(parley::StyleProperty::LineHeight(
                    widget_style
                        .line_height
                        .map(|lh| widget_style.font_size / lh)
                        .unwrap_or(1.2),
                ));
                styles.insert(parley::StyleProperty::FontStack(parley::FontStack::Single(
                    parley::FontFamily::Named(
                        font_manager
                            .get_family(widget_style.font.as_ref().unwrap_or(&default_font.0.id()))
                            .into(),
                    ),
                )));

                styles.insert(parley::StyleProperty::OverflowWrap(
                    match widget_style.text_wrap {
                        crate::styles::TextWrap::None => parley::OverflowWrap::Normal,
                        crate::styles::TextWrap::Glyph => parley::OverflowWrap::Anywhere,
                        crate::styles::TextWrap::Word => parley::OverflowWrap::BreakWord,
                        crate::styles::TextWrap::WordOrGlyph => parley::OverflowWrap::Anywhere,
                    },
                ));
                layout_editor.set_width(Some(parent_layout.size.x * camera_scale.x));
                let alignment = match widget_style
                    .text_alignment
                    .unwrap_or(crate::font::TextAlign::Left)
                {
                    crate::font::TextAlign::Left => parley::Alignment::Left,
                    crate::font::TextAlign::Right => parley::Alignment::Right,
                    crate::font::TextAlign::Center => parley::Alignment::Middle,
                    crate::font::TextAlign::Justified => parley::Alignment::Justified,
                    crate::font::TextAlign::End => parley::Alignment::End,
                };
                layout_editor.set_alignment(alignment);
                let text_layout =
                    layout_editor.layout(&mut font_manager.font_cx, &mut font_manager.layout_cx);

                for line in text_layout.lines() {
                    for item in line.items() {
                        let parley::PositionedLayoutItem::GlyphRun(glyph_run) = item else {
                            continue;
                        };

                        let mut x = glyph_run.offset();
                        let y = glyph_run.baseline();
                        let run = glyph_run.run();
                        let font = run.font();
                        let font_size = run.font_size();
                        let synthesis = run.synthesis();

                        let posx = location_x;
                        let posy = location_y;

                        // Culling
                        let mut glyph_xform = synthesis
                            .skew()
                            .map(|angle| Affine::skew(angle.to_radians().tan() as f64, 0.0))
                            .unwrap_or_else(vello::kurbo::Affine::default);

                        let trans = glyph_xform.translation();
                        glyph_xform = glyph_xform.with_translation(
                            trans + bevy_vello::vello::kurbo::Vec2::new(posx as f64, posy as f64),
                        );

                        let color = widget_style.color.to_srgba();

                        vello_scene
                            .draw_glyphs(font)
                            .hint(true)
                            .font_size(font_size * camera_scale.x)
                            .transform(glyph_xform)
                            .normalized_coords(run.normalized_coords())
                            .brush(&Brush::Solid(vello::peniko::Color::new([
                                color.red,
                                color.green,
                                color.blue,
                                color.alpha,
                            ])))
                            .draw(
                                vello::peniko::Fill::NonZero,
                                glyph_run.glyphs().map(|glyph| {
                                    let gx = x + glyph.x;
                                    let gy = y - glyph.y;
                                    x += glyph.advance;
                                    vello::Glyph {
                                        id: glyph.id as _,
                                        x: gx,
                                        y: gy,
                                    }
                                }),
                            );
                    }
                }
            }
            WidgetRender::Custom { render } => {
                render.render(vello_scene, layout, widget_style, camera_scale.x);
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

                let scale = fit_image(image.size().as_vec2(), Vec2::new(size_x, size_y)) as f64;

                let transform = vello::kurbo::Affine::scale(scale).with_translation(
                    bevy_vello::prelude::kurbo::Vec2::new(location_x as f64, location_y as f64),
                );

                let image_quality = widget_style.image_quality.into();
                let vello_image = image_manager
                    .images
                    .entry(image_handle.into())
                    .or_insert_with(move || {
                        let mut image = peniko::Image::new(
                            image.data.clone().unwrap().into(), // TODO: Don't unwrap here.
                            peniko::ImageFormat::Rgba8,
                            image.size().x,
                            image.size().y,
                        );
                        image.quality = image_quality;
                        image
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
                    Vec2::new(size_x, size_y),
                ) as f64)
                .with_translation(bevy_vello::prelude::kurbo::Vec2::new(
                    location_x as f64,
                    location_y as f64,
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
                let layout_size = Vec2::new(size_x, size_y);
                let slices = match scale_mode {
                    SpriteImageMode::Auto => {
                        todo!("Not supported yet!");
                    }
                    SpriteImageMode::Sliced(slicer) => {
                        slicer.compute_slices(image_rect, Some(layout_size))
                    }
                    SpriteImageMode::Tiled {
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
                    SpriteImageMode::Scale(_) => todo!("Not supported yet!"),
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
                            image.data.clone().unwrap().clone(), // TODO: replace unwrap with continue/return.
                        )
                        .unwrap();
                        let mut image: image::DynamicImage = image::DynamicImage::ImageRgba8(image);
                        let sub_section_data =
                            subsection_image_data(&mut image, texture_rect_floor);
                        let vello_image = peniko::Image::new(
                            sub_section_data.into(),
                            peniko::ImageFormat::Rgba8,
                            texture_rect_floor.size().x as u32,
                            texture_rect_floor.size().y as u32,
                        );
                        image_manager.nine_patch_slices.insert(key, vello_image);
                    }

                    let vello_image = image_manager.nine_patch_slices.get(&key).unwrap();
                    let scale = slice.draw_size / texture_rect_floor.size();
                    let pos = (
                        slice.offset.x.round() + (size_x / 2.0),
                        -slice.offset.y.round() + (size_y / 2.0),
                    );

                    let transform =
                        vello::kurbo::Affine::scale_non_uniform(scale.x as f64, scale.y as f64)
                            .with_translation(bevy_vello::prelude::kurbo::Vec2::new(
                                (location_x as f64 + pos.0 as f64)
                                    - (slice.draw_size.x as f64 / 2.0),
                                (location_y as f64 + pos.1 as f64)
                                    - (slice.draw_size.y as f64 / 2.0),
                            ));

                    vello_scene.draw_image(vello_image, transform);
                }
            }
            WidgetRender::RenderTarget { handle } => {
                let Some(image) = image_assets.get(handle) else {
                    return false;
                };
                let image_texture_descriptor = image.texture_descriptor.clone();

                let scale = fit_image(
                    Vec2::new(
                        image_texture_descriptor.size.width as f32,
                        image_texture_descriptor.size.height as f32,
                    ),
                    Vec2::new(size_x, size_y),
                ) as f64;

                let transform = vello::kurbo::Affine::scale(scale).with_translation(
                    bevy_vello::prelude::kurbo::Vec2::new(location_x as f64, location_y as f64),
                );

                if !render_targets.images.contains_key(handle) {
                    let mut conv_image = Image::new_uninit(
                        image_texture_descriptor.size,
                        image_texture_descriptor.dimension,
                        TextureFormat::Rgba8Unorm,
                        RenderAssetUsages::RENDER_WORLD,
                    );
                    conv_image.texture_descriptor.usage =
                        TextureUsages::COPY_SRC | TextureUsages::RENDER_ATTACHMENT;
                    let conv_image_handle = image_assets.add(conv_image);

                    render_targets
                        .images
                        .insert(handle.clone(), conv_image_handle);
                    let data: Vec<u8> = vec![];
                    render_targets.vello_images.insert(
                        handle.clone(),
                        peniko::Image::new(
                            data.into(),
                            peniko::ImageFormat::Rgba8,
                            image_texture_descriptor.size.width,
                            image_texture_descriptor.size.height,
                        ),
                    );
                }
                let vello_image = render_targets.vello_images.get(handle).unwrap();
                vello_scene.draw_image(vello_image, transform);
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
    inner: Arc<dyn Fn(&mut VelloScene, &WidgetLayout, &WoodpeckerStyle, f32) + Send + Sync>,
}

impl Default for WidgetRenderCustom {
    fn default() -> Self {
        Self::new(|_, _, _, _| {})
    }
}

impl WidgetRenderCustom {
    /// Create a new custom widget render.
    pub fn new<F>(render: F) -> Self
    where
        F: Fn(&mut VelloScene, &WidgetLayout, &WoodpeckerStyle, f32) + Send + Sync + 'static,
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
        dpi: f32,
    ) {
        self.inner.clone()(vello_scene, layout, styles, dpi);
    }
}
