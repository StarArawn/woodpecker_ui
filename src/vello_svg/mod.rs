#![allow(unused)]
// Copyright 2023 the Vello Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Render an SVG document to a Vello [`Scene`].
//!
//! This currently lacks support for a [number of important](crate#unsupported-features) SVG features.
//!
//! This is also intended to be the preferred integration between Vello and [usvg], so [consider
//! contributing](https://github.com/linebender/vello_svg) if you need a feature which is missing.
//!
//! This crate also re-exports [`usvg`] and [`vello`], so you can easily use the specific versions that are compatible with Vello SVG.
//!
//! # Unsupported features
//!
//! Missing features include:
//! - text
//! - group opacity
//! - mix-blend-modes
//! - clipping
//! - masking
//! - filter effects
//! - group background
//! - path shape-rendering
//! - patterns

mod render;

mod error;
use error::Error;

mod util;

use bevy_vello::vello::{kurbo::Affine, peniko::Color};

/// Render a [`Scene`] from an SVG string, with default error handling.
///
/// This will draw a red box over (some) unsupported elements.
pub(crate) fn render(
    svg: &str,
    path_color: Option<Color>,
) -> Result<bevy_vello::vello::Scene, Error> {
    let opt = usvg::Options::default();
    let tree = usvg::Tree::from_str(svg, &opt)?;
    let mut scene = bevy_vello::vello::Scene::new();
    append_tree(&mut scene, &tree, path_color);
    Ok(scene)
}

/// Append an SVG to a vello [`Scene`], with default error handling.
///
/// This will draw a red box over (some) unsupported elements.
pub(crate) fn append(scene: &mut bevy_vello::vello::Scene, svg: &str) -> Result<(), Error> {
    let opt = usvg::Options::default();
    let tree = usvg::Tree::from_str(svg, &opt)?;
    append_tree(scene, &tree, None);
    Ok(())
}

/// Append an SVG to a vello [`Scene`], with user-provided error handling logic.
///
/// See the [module level documentation](crate#unsupported-features) for a list of some unsupported svg features
pub(crate) fn append_with<F: FnMut(&mut bevy_vello::vello::Scene, &usvg::Node)>(
    scene: &mut bevy_vello::vello::Scene,
    svg: &str,
    error_handler: &mut F,
) -> Result<(), Error> {
    let opt = usvg::Options::default();
    let tree = usvg::Tree::from_str(svg, &opt)?;
    append_tree_with(scene, &tree, None, error_handler);
    Ok(())
}

/// Render a [`Scene`] from a [`usvg::Tree`], with default error handling.
///
/// This will draw a red box over (some) unsupported elements.
pub(crate) fn render_tree(svg: &usvg::Tree, path_color: Option<Color>) -> bevy_vello::vello::Scene {
    let mut scene = bevy_vello::vello::Scene::new();
    append_tree(&mut scene, svg, path_color);
    scene
}

/// Append an [`usvg::Tree`]  to a vello [`Scene`], with default error handling.
///
/// This will draw a red box over (some) unsupported elements.
pub(crate) fn append_tree(
    scene: &mut bevy_vello::vello::Scene,
    svg: &usvg::Tree,
    color: Option<Color>,
) {
    append_tree_with(scene, svg, color, &mut util::default_error_handler);
}

/// Append an [`usvg::Tree`] to a vello [`Scene`], with user-provided error handling logic.
///
/// See the [module level documentation](crate#unsupported-features) for a list of some unsupported svg features
pub(crate) fn append_tree_with<F: FnMut(&mut bevy_vello::vello::Scene, &usvg::Node)>(
    scene: &mut bevy_vello::vello::Scene,
    svg: &usvg::Tree,
    color: Option<Color>,
    error_handler: &mut F,
) {
    render::render_group(scene, svg.root(), color, Affine::IDENTITY, error_handler);
}
