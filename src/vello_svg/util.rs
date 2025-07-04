// Copyright 2023 the Vello Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

use bevy_vello::vello::kurbo::{Affine, BezPath, Point, Rect, Stroke};
use bevy_vello::vello::peniko::color::DynamicColor;
use bevy_vello::vello::peniko::{Blob, Brush, Color, Fill, Image};
use bevy_vello::vello::Scene;

pub fn to_affine(ts: &usvg::Transform) -> Affine {
    let usvg::Transform {
        sx,
        kx,
        ky,
        sy,
        tx,
        ty,
    } = ts;
    Affine::new([sx, kx, ky, sy, tx, ty].map(|&x| f64::from(x)))
}

pub fn to_stroke(stroke: &usvg::Stroke) -> Stroke {
    let mut conv_stroke = Stroke::new(stroke.width().get() as f64)
        .with_caps(match stroke.linecap() {
            usvg::LineCap::Butt => bevy_vello::vello::kurbo::Cap::Butt,
            usvg::LineCap::Round => bevy_vello::vello::kurbo::Cap::Round,
            usvg::LineCap::Square => bevy_vello::vello::kurbo::Cap::Square,
        })
        .with_join(match stroke.linejoin() {
            usvg::LineJoin::Miter | usvg::LineJoin::MiterClip => {
                bevy_vello::vello::kurbo::Join::Miter
            }
            usvg::LineJoin::Round => bevy_vello::vello::kurbo::Join::Round,
            usvg::LineJoin::Bevel => bevy_vello::vello::kurbo::Join::Bevel,
        })
        .with_miter_limit(stroke.miterlimit().get() as f64);
    if let Some(dash_array) = stroke.dasharray().as_ref() {
        conv_stroke = conv_stroke.with_dashes(
            stroke.dashoffset() as f64,
            dash_array.iter().map(|x| *x as f64),
        );
    }
    conv_stroke
}

pub fn to_bez_path(path: &usvg::Path) -> BezPath {
    let mut local_path = BezPath::new();
    // The semantics of SVG paths don't line up with `BezPath`; we
    // must manually track initial points
    let mut just_closed = false;
    let mut most_recent_initial = (0., 0.);
    for elt in path.data().segments() {
        match elt {
            usvg::tiny_skia_path::PathSegment::MoveTo(p) => {
                if std::mem::take(&mut just_closed) {
                    local_path.move_to(most_recent_initial);
                }
                most_recent_initial = (p.x.into(), p.y.into());
                local_path.move_to(most_recent_initial);
            }
            usvg::tiny_skia_path::PathSegment::LineTo(p) => {
                if std::mem::take(&mut just_closed) {
                    local_path.move_to(most_recent_initial);
                }
                local_path.line_to(Point::new(p.x as f64, p.y as f64));
            }
            usvg::tiny_skia_path::PathSegment::QuadTo(p1, p2) => {
                if std::mem::take(&mut just_closed) {
                    local_path.move_to(most_recent_initial);
                }
                local_path.quad_to(
                    Point::new(p1.x as f64, p1.y as f64),
                    Point::new(p2.x as f64, p2.y as f64),
                );
            }
            usvg::tiny_skia_path::PathSegment::CubicTo(p1, p2, p3) => {
                if std::mem::take(&mut just_closed) {
                    local_path.move_to(most_recent_initial);
                }
                local_path.curve_to(
                    Point::new(p1.x as f64, p1.y as f64),
                    Point::new(p2.x as f64, p2.y as f64),
                    Point::new(p3.x as f64, p3.y as f64),
                );
            }
            usvg::tiny_skia_path::PathSegment::Close => {
                just_closed = true;
                local_path.close_path();
            }
        }
    }

    local_path
}

pub fn into_image(image: image::ImageBuffer<image::Rgba<u8>, Vec<u8>>) -> Image {
    let (width, height) = (image.width(), image.height());
    let image_data: Vec<u8> = image.into_vec();
    Image::new(
        Blob::new(std::sync::Arc::new(image_data)),
        bevy_vello::vello::peniko::ImageFormat::Rgba8,
        width,
        height,
    )
}

pub fn to_brush(
    paint: &usvg::Paint,
    opacity: usvg::Opacity,
    path_color: Option<Color>,
) -> Option<(Brush, Affine)> {
    if let Some(path_color) = path_color {
        return Some((Brush::Solid(path_color), Affine::IDENTITY));
    }

    match paint {
        usvg::Paint::Color(color) => Some((
            Brush::Solid(Color::from_rgba8(
                color.red,
                color.green,
                color.blue,
                opacity.to_u8(),
            )),
            Affine::IDENTITY,
        )),
        usvg::Paint::LinearGradient(gr) => {
            let stops: Vec<bevy_vello::vello::peniko::ColorStop> = gr
                .stops()
                .iter()
                .map(|stop| bevy_vello::vello::peniko::ColorStop {
                    offset: stop.offset().get(),
                    color: DynamicColor::from_alpha_color(
                        bevy_vello::vello::peniko::Color::from_rgba8(
                            stop.color().red,
                            stop.color().green,
                            stop.color().blue,
                            (stop.opacity() * opacity).to_u8(),
                        ),
                    ),
                })
                .collect();
            let start = Point::new(gr.x1() as f64, gr.y1() as f64);
            let end = Point::new(gr.x2() as f64, gr.y2() as f64);
            let arr = [
                gr.transform().sx,
                gr.transform().ky,
                gr.transform().kx,
                gr.transform().sy,
                gr.transform().tx,
                gr.transform().ty,
            ]
            .map(f64::from);
            let transform = Affine::new(arr);
            let gradient = bevy_vello::vello::peniko::Gradient::new_linear(start, end)
                .with_stops(stops.as_slice());
            Some((Brush::Gradient(gradient), transform))
        }
        usvg::Paint::RadialGradient(gr) => {
            let stops: Vec<bevy_vello::vello::peniko::ColorStop> = gr
                .stops()
                .iter()
                .map(|stop| bevy_vello::vello::peniko::ColorStop {
                    offset: stop.offset().get(),
                    color: DynamicColor::from_alpha_color(
                        bevy_vello::vello::peniko::Color::from_rgba8(
                            stop.color().red,
                            stop.color().green,
                            stop.color().blue,
                            (stop.opacity() * opacity).to_u8(),
                        ),
                    ),
                })
                .collect();

            let start_center = Point::new(gr.cx() as f64, gr.cy() as f64);
            let end_center = Point::new(gr.fx() as f64, gr.fy() as f64);
            let start_radius = 0_f32;
            let end_radius = gr.r().get();
            let arr = [
                gr.transform().sx,
                gr.transform().ky,
                gr.transform().kx,
                gr.transform().sy,
                gr.transform().tx,
                gr.transform().ty,
            ]
            .map(f64::from);
            let transform = Affine::new(arr);
            let gradient = bevy_vello::vello::peniko::Gradient::new_two_point_radial(
                start_center,
                start_radius,
                end_center,
                end_radius,
            )
            .with_stops(stops.as_slice());
            Some((Brush::Gradient(gradient), transform))
        }
        usvg::Paint::Pattern(_) => None,
    }
}

/// Error handler function for [`super::render_tree_with`] which draws a transparent red box
/// instead of unsupported SVG features
pub fn default_error_handler(scene: &mut Scene, node: &usvg::Node) {
    let bb = node.bounding_box();
    let rect = Rect {
        x0: bb.left() as f64,
        y0: bb.top() as f64,
        x1: bb.right() as f64,
        y1: bb.bottom() as f64,
    };
    scene.fill(
        Fill::NonZero,
        Affine::IDENTITY,
        Color::new([1.0, 0.0, 0.0, 0.5]),
        None,
        &rect,
    );
}

pub fn decode_raw_raster_image(
    img: &usvg::ImageKind,
) -> Result<image::RgbaImage, image::ImageError> {
    let res = match img {
        usvg::ImageKind::JPEG(data) => {
            image::load_from_memory_with_format(data, image::ImageFormat::Jpeg)
        }
        usvg::ImageKind::PNG(data) => {
            image::load_from_memory_with_format(data, image::ImageFormat::Png)
        }
        usvg::ImageKind::GIF(data) => {
            image::load_from_memory_with_format(data, image::ImageFormat::Gif)
        }
        usvg::ImageKind::WEBP(data) => {
            image::load_from_memory_with_format(data, image::ImageFormat::WebP)
        }
        usvg::ImageKind::SVG(_) => unreachable!(),
    }?
    .into_rgba8();
    Ok(res)
}
