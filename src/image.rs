use bevy::{platform::collections::HashMap, prelude::*};
use bevy_vello::vello::peniko;

#[derive(Resource, Default)]
pub struct ImageManager {
    pub images: HashMap<AssetId<Image>, peniko::Image>,
    pub nine_patch_slices: HashMap<u64, peniko::Image>,
}
