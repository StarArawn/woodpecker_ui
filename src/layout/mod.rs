pub(crate) mod measure;
pub(crate) mod system;

use bevy::{ecs::entity::EntityHashMap, prelude::*};
use measure::{LayoutMeasure, Measure};
use taffy::{Size, TaffyTree};

use crate::{
    has_root,
    prelude::{GridTemplate, WoodpeckerStyle},
};

pub(crate) struct WoodpeckerLayoutPlugin;
impl Plugin for WoodpeckerLayoutPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UiLayout>().add_systems(
            PostUpdate,
            system::run.after(crate::runner::system).run_if(has_root()),
        );
    }
}

#[derive(Resource)]
pub(crate) struct UiLayout {
    pub(crate) root_entity: Entity,
    entity_to_taffy: EntityHashMap<taffy::NodeId>,
    taffy: TaffyTree<LayoutMeasure>,
    /// The root viewport size as of the last frame `system::run` actually recomputed
    /// layout -- lets the skip-if-clean fast path detect a window resize (which changes
    /// nothing any `Changed<T>` query would catch, since it's read fresh from the root
    /// widget's own style every frame, not mutated) as one more dirty signal.
    pub(crate) last_root_size: Option<Vec2>,
    /// For each `Text`/`RichText` entity, the parent width its `LayoutMeasure` was last
    /// computed against -- lets `system::traverse_upsert_node`'s incremental gating detect a
    /// parent resize (which touches no `Changed<T>` on the *text* entity itself) as a reason
    /// to remeasure. See that function's doc comment for why only text needs this and
    /// `Image`/`Svg` don't.
    pub(crate) previous_measure_width: EntityHashMap<f32>,
    /// Whether this frame's layout pass actually ran (as opposed to taking the skip-if-clean
    /// fast path) -- lets `vello_renderer::run` reuse the exact same signal to skip rebuilding
    /// the vello `Scene` on an idle frame, instead of recomputing it a second time. Note this
    /// is *not* simply "something changed this frame" -- see
    /// `system::run`'s `settled_frames_in_a_row` for why a still-settling frame counts as
    /// dirty here too.
    pub(crate) dirty_this_frame: bool,
    /// How many consecutive frames `system::run` has found nothing dirty. Only once this
    /// reaches [`system::SETTLE_FRAMES`] does the skip-if-clean fast path actually trust that
    /// the tree is done settling and start taking it -- see that constant's doc comment for
    /// why a single clean-looking frame isn't enough on its own.
    pub(crate) settled_frames_in_a_row: u32,
}

impl Default for UiLayout {
    fn default() -> Self {
        Self {
            root_entity: Entity::PLACEHOLDER,
            entity_to_taffy: Default::default(),
            taffy: TaffyTree::new(),
            last_root_size: None,
            previous_measure_width: Default::default(),
            dirty_this_frame: true,
            settled_frames_in_a_row: 0,
        }
    }
}

impl UiLayout {
    /// Retrieves the Taffy node associated with the given UI node entity and updates its style.
    /// If no associated Taffy node exists a new Taffy node is inserted into the Taffy layout.
    ///
    /// `grid_template` should be `Some` when the entity has a [`GridTemplate`] component
    /// (i.e. it's a grid container) -- its track definitions are overlaid onto the
    /// `taffy::Style` built from `style` alone, since `GridTemplate` lives in a separate
    /// component (see its doc comment for why).
    pub fn upsert_node(
        &mut self,
        entity: Entity,
        style: &WoodpeckerStyle,
        grid_template: Option<&GridTemplate>,
        mut new_node_context: Option<LayoutMeasure>,
    ) {
        let taffy = &mut self.taffy;

        let taffy_style = || {
            let mut taffy_style: taffy::Style = style.into();
            if let Some(grid_template) = grid_template {
                grid_template.apply(&mut taffy_style);
            }
            taffy_style
        };

        let mut added = false;
        let taffy_node_id = *self.entity_to_taffy.entry(entity).or_insert_with(|| {
            added = true;
            if let Some(measure) = new_node_context.take() {
                taffy.new_leaf_with_context(taffy_style(), measure).unwrap()
            } else {
                taffy.new_leaf(taffy_style()).unwrap()
            }
        });

        if !added {
            if new_node_context.is_some() {
                taffy
                    .set_node_context(taffy_node_id, new_node_context)
                    .unwrap();
            }

            taffy.set_style(taffy_node_id, taffy_style()).unwrap();
        }
    }

    pub fn add_child(&mut self, parent: Entity, child: Entity) {
        if !self.entity_to_taffy.contains_key(&parent) {
            return;
        }
        let parent_node_id = *self.entity_to_taffy.get(&parent).unwrap();

        if !self.entity_to_taffy.contains_key(&child) {
            return;
        }
        let child_node_id = *self.entity_to_taffy.get(&child).unwrap();

        // `taffy::TaffyTree::add_child` doesn't dedupe, so re-adding an already-attached
        // pair (e.g. a re-hoisted `position: Fixed` widget) would create duplicate edges
        // that corrupt layout and the picking-priority `order` counter.
        if self.taffy.parent(child_node_id) == Some(parent_node_id) {
            return;
        }

        self.taffy.add_child(parent_node_id, child_node_id).unwrap();
    }

    /// Detaches `child` from `parent` without removing `child`'s taffy node entirely
    /// (unlike [`Self::remove_child`]). A no-op if `child` isn't currently attached to
    /// `parent`.
    pub fn detach_child(&mut self, parent: Entity, child: Entity) {
        let Some(parent_node_id) = self.entity_to_taffy.get(&parent).copied() else {
            return;
        };
        let Some(child_node_id) = self.entity_to_taffy.get(&child).copied() else {
            return;
        };
        if self.taffy.parent(child_node_id) != Some(parent_node_id) {
            return;
        }
        let _ = self.taffy.remove_child(parent_node_id, child_node_id);
    }

    pub fn remove_child(&mut self, entity: Entity) {
        if let Some(node_id) = self.entity_to_taffy.remove(&entity) {
            let _ = self.taffy.remove(node_id);
        }
        self.previous_measure_width.remove(&entity);
    }

    pub fn add_children(&mut self, entity: Entity, children: &[Entity]) {
        if !self.entity_to_taffy.contains_key(&entity) {
            return;
        }
        let node_id = self.entity_to_taffy.get(&entity).unwrap();
        let children = children
            .iter()
            .map(|child| *self.entity_to_taffy.get(child).unwrap_or_else(|| panic!("Woodpecker UI: Couldn't find the child entity layout for {:?}. \
                This likely occurred because you are missing a WoodpeckerStyle component on one of your widgets", child)))
            .collect::<Vec<_>>();
        self.taffy.set_children(*node_id, &children).unwrap();
    }

    /// Get the layout geometry for the taffy node corresponding to the ui node [`Entity`].
    pub fn get_layout(&self, entity: Entity) -> Option<&taffy::Layout> {
        if let Some(taffy_node) = self.entity_to_taffy.get(&entity) {
            self.taffy.layout(*taffy_node).ok()
        } else {
            if entity != self.root_entity {
                trace!(
                    "Styled child in a non-UI entity hierarchy. You are using an entity \
    with UI components as a child of an entity without UI components, results may be unexpected."
                );
            }
            None
        }
    }

    pub fn compute(&mut self, root_node: Entity, root_node_size: Vec2) {
        let Some(root_id) = self.entity_to_taffy.get(&root_node) else {
            return;
        };
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
        // For debugging uncomment next line..
        // self.taffy.print_tree(*root_id);
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
