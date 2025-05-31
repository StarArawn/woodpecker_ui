use bevy::{platform::collections::HashMap, prelude::*, render::extract_resource::ExtractResource};
use bevy_vello::vello::peniko;

#[derive(Resource, Default, ExtractResource, Clone)]
pub struct ImageManager {
    pub images: HashMap<AssetId<Image>, peniko::Image>,
    pub nine_patch_slices: HashMap<u64, peniko::Image>,
}
