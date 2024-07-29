use bevy::{prelude::*, utils::HashMap};
use bevy_vello::vello::peniko;

#[derive(Resource, Default)]
pub struct ImageManager {
    pub images: HashMap<AssetId<Image>, peniko::Image>,
}