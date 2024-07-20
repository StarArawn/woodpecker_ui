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
        kurbo::{self, Affine, RoundedRectRadii},
        peniko::{self, Brush},
    },
    VelloScene,
};

use crate::prelude::WoodpeckerStyle;

pub(crate) const VARIATIONS: &[(&str, f32)] = &[];

#[derive(Component, Clone)]
pub enum WidgetRender {
    Quad,
    Text {
        // TODO: Move some of this to styles..
        font: Handle<VelloFont>,
        alignment: VelloTextAlignment,
        content: String,
        word_wrap: bool,
    },
    Custom {
        render: WidgetRenderCustom,
    },
    Layer,
}

impl WidgetRender {
    pub(crate) fn render(
        &self,
        vello_scene: &mut VelloScene,
        layout: &taffy::Layout,
        font_assets: &Assets<VelloFont>,
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
            }
            WidgetRender::Text {
                font,
                content,
                alignment,
                word_wrap,
            } => {
                let Some(font_asset) = font_assets.get(font) else {
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
                let line_height = metrics.ascent - metrics.descent + metrics.leading;
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
                    let height = (metrics.cap_height.unwrap_or(line_height) + pen_y) as f64;
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
