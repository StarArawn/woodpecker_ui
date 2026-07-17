use crate::prelude::*;
use bevy::prelude::*;

fn overlay_root_style() -> WoodpeckerStyle {
    WoodpeckerStyle {
        position: WidgetPosition::Fixed,
        width: Units::Percentage(100.0),
        height: Units::Percentage(100.0),
        top: 0.0.into(),
        left: 0.0.into(),
        ..Default::default()
    }
}

/// The shared physical home for every `.portal()`-ed built-in overlay widget (`Modal`,
/// `ToastViewport`, and friends -- see each widget's own doc comment for whether it's migrated
/// yet). Place exactly one of these near your app's root, the same way you'd place a
/// `ToastViewport`/`WindowingContextProvider` -- it's opt-in and must go through normal
/// `WidgetChildren` reconciliation to exist as a real entity; the crate can't auto-inject it
/// (a raw `commands.spawn` bypassing reconciliation would get silently despawned the next time
/// its declaring parent re-renders). Once mounted, [`sync_overlay_root`] points the shared
/// [`OverlayRoot`] resource at it, and every `.portal()`/`.portal_to(overlay_root.0)` call
/// resolves through that resource from then on.
#[derive(Widget, Component, Reflect, Clone, Default, PartialEq)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(render)]
#[require(WoodpeckerStyle = overlay_root_style(), WidgetChildren, StackingContext)]
pub struct OverlayRootWidget;

fn render(
    entity: Res<CurrentWidget>,
    mut query: Query<&mut WidgetChildren, With<OverlayRootWidget>>,
) {
    let Ok(mut children) = query.get_mut(**entity) else {
        return;
    };
    children.apply(entity.as_parent());
}

/// One-time bootstrap: as soon as `OverlayRootWidget` is actually spawned (via normal
/// `WidgetChildren` reconciliation, not before), point the shared [`OverlayRoot`] resource at
/// it. Registered in `PreUpdate` by [`super::WoodpeckerUIWidgetPlugin`], ahead of the `Update`-
/// stage widget tree dispatch, so any portaled widget's very first render already sees it --
/// portaled widgets still check `Option<Res<OverlayRoot>>` and skip a frame rather than panic,
/// since a widget declared as a sibling of `OverlayRootWidget` in the same initial tree can in
/// principle still render before this has run once.
pub(crate) fn sync_overlay_root(
    query: Query<Entity, Added<OverlayRootWidget>>,
    mut commands: Commands,
) {
    if let Ok(entity) = query.single() {
        commands.insert_resource(OverlayRoot(entity));
    }
}
