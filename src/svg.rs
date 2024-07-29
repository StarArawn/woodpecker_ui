use std::{
    hash::{DefaultHasher, Hash, Hasher},
    sync::Arc,
};

use bevy::{
    asset::{io::Reader, AssetLoader, AsyncReadExt, LoadContext},
    prelude::*,
    utils::{ConditionalSendFuture, HashMap},
};
use bevy_vello::{integrations::VectorLoaderError, vello_svg};

#[derive(Default)]
pub struct SvgLoader;

#[derive(Asset, TypePath, Clone)]
pub struct SvgAsset {
    pub tree: usvg::Tree,
    pub width: f32,
    pub height: f32,
}

impl AssetLoader for SvgLoader {
    type Asset = SvgAsset;

    type Settings = ();

    type Error = VectorLoaderError;

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _settings: &'a Self::Settings,
        load_context: &'a mut LoadContext,
    ) -> impl ConditionalSendFuture<Output = Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;
            let path = load_context.path().to_owned();
            let ext =
                path.extension()
                    .and_then(std::ffi::OsStr::to_str)
                    .ok_or(VectorLoaderError::Io(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Invalid file extension",
                    )))?;

            debug!("parsing {}...", load_context.path().display());
            match ext {
                "svg" => {
                    let svg_str = std::str::from_utf8(&bytes)?;
                    let svg = usvg::Tree::from_str(svg_str, &usvg::Options::default())
                        .map_err(vello_svg::Error::Svg)?;
                    let width = svg.size().width();
                    let height = svg.size().height();
                    Ok(SvgAsset {
                        tree: svg,
                        width,
                        height,
                    })
                }
                ext => Err(VectorLoaderError::Io(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Invalid file extension: '{ext}'"),
                ))),
            }
        })
    }

    fn extensions(&self) -> &[&str] {
        &["svg"]
    }
}

#[derive(Resource, Default)]
pub struct SvgManager {
    svg_cache: HashMap<u64, Arc<bevy_vello::vello::Scene>>,
}

impl SvgManager {
    pub fn get_cached(
        &mut self,
        asset_id: impl Into<AssetId<SvgAsset>>,
        svg_assets: &Assets<SvgAsset>,
        color: Option<Color>,
    ) -> Option<Arc<bevy_vello::vello::Scene>> {
        let asset_id: AssetId<SvgAsset> = asset_id.into();
        let svg = svg_assets.get(asset_id)?;

        let mut hasher = DefaultHasher::default();
        asset_id.hash(&mut hasher);
        if let Some(color) = color {
            color.to_srgba().to_hex().hash(&mut hasher);
        }
        let key: u64 = hasher.finish();

        Some(
            self.svg_cache
                .entry(key)
                .or_insert_with(|| {
                    Arc::new(crate::vello_svg::render_tree(
                        &svg.tree,
                        color.map(|c| {
                            let c = c.to_srgba();
                            bevy_vello::vello::peniko::Color::rgba(
                                c.red as f64,
                                c.green as f64,
                                c.blue as f64,
                                c.alpha as f64,
                            )
                        }),
                    ))
                })
                .clone(),
        )
    }
}
