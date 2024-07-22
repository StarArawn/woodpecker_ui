mod measure;
pub(crate) mod system;

use bevy::{prelude::*, utils::EntityHashMap};
use measure::{LayoutMeasure, Measure};
use taffy::{Size, TaffyTree};

use crate::{has_root, prelude::WoodpeckerStyle};

pub struct WoodpeckerLayoutPlugin;
impl Plugin for WoodpeckerLayoutPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UiLayout>().add_systems(
            Update,
            system::run.after(crate::runner::system).run_if(has_root()),
        );
    }
}

#[derive(Resource)]
pub struct UiLayout {
    entity_to_taffy: EntityHashMap<Entity, taffy::NodeId>,
    taffy: TaffyTree<LayoutMeasure>,
}

impl Default for UiLayout {
    fn default() -> Self {
        Self {
            entity_to_taffy: Default::default(),
            taffy: TaffyTree::new(),
        }
    }
}

impl UiLayout {
    /// Retrieves the Taffy node associated with the given UI node entity and updates its style.
    /// If no associated Taffy node exists a new Taffy node is inserted into the Taffy layout.
    pub fn upsert_node(
        &mut self,
        entity: Entity,
        style: &WoodpeckerStyle,
        mut new_node_context: Option<LayoutMeasure>,
    ) {
        let taffy = &mut self.taffy;

        let mut added = false;
        let taffy_node_id = *self.entity_to_taffy.entry(entity).or_insert_with(|| {
            added = true;
            if let Some(measure) = new_node_context.take() {
                taffy.new_leaf_with_context(style.into(), measure).unwrap()
            } else {
                taffy.new_leaf(style.into()).unwrap()
            }
        });

        if !added {
            // let has_measure = if new_node_context.is_some() {
            //     taffy
            //         .set_node_context(taffy_node_id, new_node_context)
            //         .unwrap();
            //     true
            // } else {
            //     taffy.get_node_context(taffy_node_id).is_some()
            // };

            taffy.set_style(taffy_node_id, style.into()).unwrap();
        }
    }

    pub fn add_child(&mut self, parent: Entity, child: Entity) {
        if !self.entity_to_taffy.contains_key(&parent) {
            return;
        }
        let parent_node_id = self.entity_to_taffy.get(&parent).unwrap();

        if !self.entity_to_taffy.contains_key(&child) {
            return;
        }
        let child_node_id = self.entity_to_taffy.get(&child).unwrap();
        self.taffy
            .add_child(*parent_node_id, *child_node_id)
            .unwrap();
    }

    pub fn add_children(&mut self, entity: Entity, children: &[Entity]) {
        if !self.entity_to_taffy.contains_key(&entity) {
            return;
        }
        let node_id = self.entity_to_taffy.get(&entity).unwrap();
        let children = children
            .iter()
            .map(|child| *self.entity_to_taffy.get(child).unwrap())
            .collect::<Vec<_>>();
        self.taffy.set_children(*node_id, &children).unwrap();
    }

    /// Get the layout geometry for the taffy node corresponding to the ui node [`Entity`].
    pub fn get_layout(&self, entity: Entity) -> Option<&taffy::Layout> {
        if let Some(taffy_node) = self.entity_to_taffy.get(&entity) {
            self.taffy.layout(*taffy_node).ok()
        } else {
            warn!(
                "Styled child in a non-UI entity hierarchy. You are using an entity \
with UI components as a child of an entity without UI components, results may be unexpected."
            );
            None
        }
    }

    pub fn compute(&mut self, root_node: Entity, root_node_size: Vec2) {
        let Some(root_id) = self.entity_to_taffy.get(&root_node) else {
            return;
        };
        // self.taffy
        //     .compute_layout(
        //         *root_id,
        //         // Size::max_content(),
        //         Size {
        //             width: taffy::AvailableSpace::Definite(root_node_size.x),
        //             height: taffy::AvailableSpace::Definite(root_node_size.y),
        //         },
        //     )
        //     .unwrap();
        // self.taffy.print_tree(*root_id);
        self.taffy
            .compute_layout_with_measure(
                *root_id,
                Size {
                    width: taffy::AvailableSpace::Definite(root_node_size.x),
                    height: taffy::AvailableSpace::Definite(root_node_size.y),
                },
                |known_dimensions: taffy::Size<Option<f32>>,
                 available_space: taffy::Size<taffy::AvailableSpace>,
                 _node_id: taffy::NodeId,
                 context: Option<&mut LayoutMeasure>,
                 style: &taffy::Style|
                 -> taffy::Size<f32> {
                    context
                        .map(|ctx| {
                            let size = ctx.measure(
                                known_dimensions.width,
                                known_dimensions.height,
                                available_space.width,
                                available_space.height,
                                style,
                            );
                            taffy::Size {
                                width: size.x,
                                height: size.y,
                            }
                        })
                        .unwrap_or(taffy::Size::ZERO)
                },
            )
            .unwrap();
    }
}

#[test]
fn test_bug() {
    use taffy::*;
    let mut taffy: TaffyTree<()> = TaffyTree::new();

    let child = taffy
        .new_leaf(Style {
            size: Size {
                width: Dimension::Percent(1.0),
                height: Dimension::Percent(1.0),
            },
            ..Default::default()
        })
        .unwrap();

    let node = taffy
        .new_with_children(
            Style {
                size: Size {
                    width: Dimension::Length(1280.0),
                    height: Dimension::Length(720.0),
                },
                justify_content: Some(JustifyContent::Center),
                ..Default::default()
            },
            &[child],
        )
        .unwrap();

    println!("Compute layout with 100x100 viewport:");
    taffy
        .compute_layout(
            node,
            Size {
                width: AvailableSpace::Definite(1280.0),
                height: AvailableSpace::Definite(720.0),
            },
        )
        .unwrap();

    taffy.print_tree(node);
}
