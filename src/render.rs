use std::sync::Arc;

use bevy::prelude::*;
use bevy_vello::{
    text::{VelloFont, VelloTextAlignment},
    vello::{
        self,
        glyph::{
            skrifa::{FontRef, MetadataProvider},
            Glyph,
        },
        kurbo::{self, Affine, Point, RoundedRectRadii, Stroke},
        peniko::{self, Brush},
    },
    VelloScene,
};

use crate::{prelude::WoodpeckerStyle, DefaultFont};

pub(crate) const VARIATIONS: &[(&str, f32)] = &[];

#[derive(Component, Clone)]
pub enum WidgetRender {
    Quad,
    Text {
        alignment: VelloTextAlignment,
        content: String,
        word_wrap: bool,
    },
    Custom {
        render: WidgetRenderCustom,
    },
    Layer,
    Image {
        image_handle: Handle<Image>,
    },
}

impl WidgetRender {
    pub(crate) fn render(
        &self,
        vello_scene: &mut VelloScene,
        layout: &taffy::Layout,
        default_font: &DefaultFont,
        font_assets: &Assets<VelloFont>,
        image_assets: &Assets<Image>,
        widget_style: &WoodpeckerStyle,
    ) -> bool {
        let mut did_layer = false;
        let location_x = layout.location.x + layout.border.left;
        let location_y = layout.location.y + layout.border.top;
        let size_x = layout.size.width - layout.border.right;
        let size_y = layout.size.height - layout.border.bottom;

        match self {
            WidgetRender::Quad => {
                let color = widget_style.background_color.to_srgba();
                let border_color = widget_style.border_color.to_srgba();
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
                if layout.border.left > 0.0 {
                    vello_scene.stroke(
                        &Stroke::new(layout.border.left as f64),
                        kurbo::Affine::default(),
                        peniko::Color::rgba(
                            border_color.red as f64,
                            border_color.green as f64,
                            border_color.blue as f64,
                            border_color.alpha as f64,
                        ),
                        None,
                        &kurbo::Line::new(
                            Point {
                                x: layout.location.x as f64 + (layout.border.left) as f64,
                                y: layout.location.y as f64,
                            },
                            Point {
                                x: layout.location.x as f64 + (layout.border.left) as f64,
                                y: layout.location.y as f64 + layout.size.height as f64,
                            },
                        ),
                    );
                }
            }
            WidgetRender::Text {
                content,
                alignment,
                word_wrap,
            } => {
                let Some(font_asset) =
                    font_assets.get(widget_style.font.as_ref().unwrap_or(&default_font.0))
                else {
                    error!("Woodpecker UI: Missing font for text: {}!", content);
                    return false;
                };
                let font =
                    FontRef::new(font_asset.font.data.data()).expect("Vello font creation error");

                let font_size = vello::skrifa::instance::Size::new(widget_style.font_size);
                let charmap = font.charmap();
                let axes = font.axes();
                let var_loc = axes.location(VARIATIONS);
                let metrics = font.metrics(font_size, &var_loc);
                let line_height = if widget_style.line_height > 0.0 {
                    // TODO: Make this an Option..
                    widget_style.line_height
                } else {
                    metrics.ascent - metrics.descent + metrics.leading
                };
                let glyph_metrics = font.glyph_metrics(font_size, &var_loc);

                let avaliable_space = layout.size.width;

                let mut pen_x = 0f32;
                let mut pen_y = 0f32;
                let mut width = 0f32;
                let glyphs: Vec<(f32, Glyph)> = content
                    .chars()
                    .filter_map(|ch| {
                        if ch == '\n' {
                            pen_y += line_height;
                            pen_x = 0.0;
                            return None;
                        }
                        let gid = charmap.map(ch).unwrap_or_default();
                        let advance = glyph_metrics.advance_width(gid).unwrap_or_default();
                        let x = pen_x;
                        pen_x += advance;
                        width = width.max(pen_x);
                        Some((
                            advance,
                            Glyph {
                                id: gid.to_u16() as u32,
                                x,
                                y: pen_y,
                            },
                        ))
                    })
                    .collect();

                // Figure out lines
                struct Line {
                    glyphs: Vec<Glyph>,
                    width: f32,
                }

                let mut line = Line {
                    glyphs: Vec::new(),
                    width: 0.0,
                };
                let mut lines = vec![];
                for (advance, glyph) in glyphs.iter() {
                    if line.width + advance >= avaliable_space && glyph.id != 3 && *word_wrap {
                        lines.push(line);
                        line = Line {
                            glyphs: Vec::new(),
                            width: 0.0,
                        };
                    }

                    line.glyphs.push(*glyph);
                    line.width += advance;
                }

                lines.push(line);

                let mut prev_line_width = 0.0;
                for (i, line) in lines.into_iter().enumerate() {
                    let mut transform = vello::kurbo::Affine::translate((
                        layout.location.x as f64 - prev_line_width,
                        layout.location.y as f64
                            + (i as f64 * metrics.cap_height.unwrap_or(line_height) as f64),
                    ));

                    // Push up from pen_y
                    transform *= vello::kurbo::Affine::translate((0.0, -pen_y as f64));

                    // Alignment settings
                    let width = width as f64;
                    let height = (if widget_style.line_height > 0.0 {
                        line_height
                    } else {
                        metrics.cap_height.unwrap_or(line_height)
                    } + pen_y) as f64;
                    match alignment {
                        VelloTextAlignment::TopLeft => {
                            transform *= vello::kurbo::Affine::translate((0.0, height));
                        }
                        VelloTextAlignment::Left => {
                            transform *= vello::kurbo::Affine::translate((0.0, height / 2.0));
                        }
                        VelloTextAlignment::BottomLeft => {
                            transform *= vello::kurbo::Affine::translate((0.0, 0.0));
                        }
                        VelloTextAlignment::Top => {
                            transform *= vello::kurbo::Affine::translate((-width / 2.0, height));
                        }
                        VelloTextAlignment::Center => {
                            transform *=
                                vello::kurbo::Affine::translate((-width / 2.0, height / 2.0));
                        }
                        VelloTextAlignment::Bottom => {
                            transform *= vello::kurbo::Affine::translate((-width / 2.0, 0.0));
                        }
                        VelloTextAlignment::TopRight => {
                            transform *= vello::kurbo::Affine::translate((-width, height));
                        }
                        VelloTextAlignment::Right => {
                            transform *= vello::kurbo::Affine::translate((-width, height / 2.0));
                        }
                        VelloTextAlignment::BottomRight => {
                            transform *= vello::kurbo::Affine::translate((-width, 0.0));
                        }
                    }

                    let color = widget_style.color.to_srgba();
                    vello_scene
                        .draw_glyphs(&font_asset.font)
                        .font_size(widget_style.font_size)
                        .transform(transform)
                        .normalized_coords(var_loc.coords())
                        .brush(&Brush::Solid(vello::peniko::Color::rgba(
                            color.red as f64,
                            color.green as f64,
                            color.blue as f64,
                            color.alpha as f64,
                        )))
                        .draw(vello::peniko::Fill::NonZero, line.glyphs.into_iter());
                    prev_line_width += line.width as f64;
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
