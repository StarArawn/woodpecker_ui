use crate::prelude::*;
use bevy::prelude::*;

/// The Woodpecker UI Element
///
/// `overflow: Clip` is load-bearing, not cosmetic: `WoodpeckerStyle::default()`'s `overflow` is
/// `Visible`, under which overflowing content still contributes to an ancestor's scroll region
/// (its own doc comment says so directly) -- so without this, `Clip` would only ever clip
/// *visually* (via `WidgetRender::Layer`'s vello scene layer) while its actual overflowing
/// content still inflated any ancestor `ScrollContent`'s measured height, defeating the entire
/// reason to reach for a clip in the first place (e.g. a bounded-height virtualized list
/// nested in a normally-scrolling page reporting its full, unclipped content height upward).
#[derive(Component, Widget, Reflect, PartialEq, Default, Clone)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(render)]
#[require(WidgetChildren, WoodpeckerStyle = WoodpeckerStyle {
    width: Units::Percentage(100.0),
    height: Units::Percentage(100.0),
    overflow: WidgetOverflow::Clip,
    ..Default::default()
}, WidgetRender::Layer)]
pub struct Clip;

pub fn render(entity: Res<CurrentWidget>, mut query: Query<&mut WidgetChildren>) {
    let Ok(mut children) = query.get_mut(**entity) else {
        return;
    };

    children.apply(entity.as_parent());
}
