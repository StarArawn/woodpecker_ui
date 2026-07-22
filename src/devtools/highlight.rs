use bevy::prelude::*;

use crate::layout::system::WidgetLayout;

/// The (position, size) of `entity`'s current [`WidgetLayout`], if it still exists -- the same
/// virtual-viewport space `WoodpeckerStyle`'s `Fixed` positioning resolves in, so it can be
/// copied straight into the highlight quad's `left`/`top`/`width`/`height`.
pub(crate) fn highlight_rect(world: &World, entity: Entity) -> Option<(Vec2, Vec2)> {
    world
        .get::<WidgetLayout>(entity)
        .map(|layout| (layout.location, layout.size))
}
