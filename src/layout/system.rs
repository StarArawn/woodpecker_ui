use bevy::{prelude::*, utils::HashMap};
use bevy_trait_query::One;
use bevy_vello::{text::VelloFont, VelloScene};
use taffy::Layout;

use crate::{
    context::{Widget, WoodpeckerContext},
    prelude::WidgetRender,
};

use super::{UiLayout, WoodpeckerStyle};

#[derive(Component, Debug, Clone, Copy, Deref, DerefMut)]
pub struct WidgetLayout(Layout);

pub(crate) fn run(
    mut commands: Commands,
    mut ui_layout: ResMut<UiLayout>,
    mut query: Query<(
        Entity,
        One<&dyn Widget>,
        &WoodpeckerStyle,
        Option<&Parent>,
        Option<&Children>,
    )>,
    children_query: Query<(Entity, &Children), Changed<Children>>,
    mut vello_query: Query<&mut VelloScene>,
    widget_render: Query<&WidgetRender>,
    context: Res<WoodpeckerContext>,
    font_assets: Res<Assets<VelloFont>>,
) {
    let Ok(mut vello_scene) = vello_query.get_single_mut() else {
        error!("Woodpecker UI: No vello scene spawned!");
        return;
    };
    vello_scene.reset();

    let root_node = context.get_root_widget();
    // This needs to be in the correct order
    // TODO: This probably doesn't need to be in the correct order..
    traverse_upsert_node(&query, &mut ui_layout, root_node);

    for (entity, children) in children_query.iter() {
        // println!("Entity: {:?} and its children: {:?}", entity, children);
        ui_layout.add_children(entity, children);
    }

    let Ok((width, height)) = query.get(root_node).map(|(_, _, style, _, _)| {
        (
            style.0.size.width.into_option().unwrap_or(0.0),
            style.0.size.height.into_option().unwrap_or(0.0),
        )
    }) else {
        return;
    };

    ui_layout.compute(root_node, Vec2::new(width, height));

    let mut cached_layout = HashMap::default();

    // After layout computations update layouts and render scene.
    // Needs to be done in the correct order..
    // We also need to know if we are going back up the tree so we can pop the clipping and opacity layers.
    traverse_render_tree(
        &mut query,
        &widget_render,
        &mut cached_layout,
        &mut vello_scene,
        &font_assets,
        &ui_layout,
        root_node,
    );

    for (entity, layout) in cached_layout.iter() {
        commands.entity(*entity).insert(WidgetLayout(*layout));
    }
}

fn traverse_render_tree(
    query: &mut Query<(
        Entity,
        One<&dyn Widget>,
        &WoodpeckerStyle,
        Option<&Parent>,
        Option<&Children>,
    )>,
    widget_render: &Query<&WidgetRender>,
    cached_layout: &mut HashMap<Entity, Layout>,
    vello_scene: &mut VelloScene,
    font_assets: &Assets<VelloFont>,
    ui_layout: &UiLayout,
    current_node: Entity,
) {
    let Ok((entity, _, _, parent, children)) = query.get_mut(current_node) else {
        return;
    };
    let Some(layout) = ui_layout.get_layout(entity).cloned() else {
        return;
    };

    let mut did_layer = false;
    if let Ok(widget_render) = widget_render.get(entity) {
        let mut layout = layout.clone();
        if let Some(parent_layout) = parent.map(|parent| {
            cached_layout
                .get(&parent.get())
                .map(|l| *l)
                .unwrap_or_else(|| *ui_layout.get_layout(parent.get()).unwrap())
        }) {
            layout.location.x += parent_layout.location.x;
            layout.location.y += parent_layout.location.y;
        }
        did_layer = widget_render.render(vello_scene, &layout, &font_assets);
        cached_layout.insert(entity, layout);
    }

    let Some(children) = children.map(|c| c.iter().map(|c| *c).collect::<Vec<_>>()) else {
        if did_layer {
            vello_scene.pop_layer();
        }
        return;
    };

    for child in children.iter() {
        traverse_render_tree(
            query,
            widget_render,
            cached_layout,
            vello_scene,
            font_assets,
            ui_layout,
            *child,
        );
    }
    if did_layer {
        vello_scene.pop_layer();
    }
}

fn traverse_upsert_node(
    query: &Query<(
        Entity,
        One<&dyn Widget>,
        &WoodpeckerStyle,
        Option<&Parent>,
        Option<&Children>,
    )>,
    layout: &mut UiLayout,
    current_node: Entity,
) {
    let Ok((entity, _, styles, _parent, children)) = query.get(current_node) else {
        return;
    };
    layout.upsert_node(entity, styles, None);
    let Some(children) = children else {
        return;
    };
    for child in children.iter() {
        traverse_upsert_node(query, layout, *child);
    }
}
