mod measure;
pub(crate) mod system;

use bevy::{prelude::*, utils::EntityHashMap};
use measure::LayoutMeasure;
use taffy::{Size, TaffyTree};

use crate::has_root;

#[derive(Component, Deref, DerefMut, Default, Debug, Clone)]
pub struct WoodpeckerStyle(taffy::Style);

impl WoodpeckerStyle {
    pub fn new() -> Self {
        Self(taffy::Style::DEFAULT)
    }

    pub fn with_display(mut self, display: taffy::Display) -> Self {
        self.display = display;
        self
    }

    pub fn with_overflow(mut self, overflow: taffy::Point<taffy::Overflow>) -> Self {
        self.overflow = overflow;
        self
    }

    pub fn with_scrollbar_width(mut self, scrollbar_width: f32) -> Self {
        self.scrollbar_width = scrollbar_width;
        self
    }

    pub fn with_position(mut self, position: taffy::Position) -> Self {
        self.position = position;
        self
    }

    pub fn with_inset(mut self, inset: taffy::Rect<taffy::LengthPercentageAuto>) -> Self {
        self.inset = inset;
        self
    }

    pub fn with_size(mut self, size: taffy::Size<taffy::Dimension>) -> Self {
        self.size = size;
        self
    }

    pub fn with_min_size(mut self, min_size: taffy::Size<taffy::Dimension>) -> Self {
        self.min_size = min_size;
        self
    }

    pub fn with_max_size(mut self, max_size: taffy::Size<taffy::Dimension>) -> Self {
        self.max_size = max_size;
        self
    }

    pub fn with_aspect_ratio(mut self, aspect_ratio: Option<f32>) -> Self {
        self.aspect_ratio = aspect_ratio;
        self
    }

    pub fn with_margin(mut self, margin: taffy::Rect<taffy::LengthPercentageAuto>) -> Self {
        self.margin = margin;
        self
    }

    pub fn with_padding(mut self, padding: taffy::Rect<taffy::LengthPercentage>) -> Self {
        self.padding = padding;
        self
    }

    pub fn with_border(mut self, border: taffy::Rect<taffy::LengthPercentage>) -> Self {
        self.border = border;
        self
    }

    pub fn with_align_items(mut self, align_items: Option<taffy::AlignItems>) -> Self {
        self.align_items = align_items;
        self
    }

    pub fn with_align_self(mut self, align_self: Option<taffy::AlignSelf>) -> Self {
        self.align_self = align_self;
        self
    }

    pub fn with_justify_items(mut self, justify_items: Option<taffy::AlignItems>) -> Self {
        self.justify_items = justify_items;
        self
    }

    pub fn with_justify_self(mut self, justify_self: Option<taffy::AlignSelf>) -> Self {
        self.justify_self = justify_self;
        self
    }

    pub fn with_align_content(mut self, align_content: Option<taffy::AlignContent>) -> Self {
        self.align_content = align_content;
        self
    }

    pub fn with_justify_content(mut self, justify_content: Option<taffy::JustifyContent>) -> Self {
        self.justify_content = justify_content;
        self
    }

    pub fn with_gap(mut self, gap: taffy::Size<taffy::LengthPercentage>) -> Self {
        self.gap = gap;
        self
    }

    pub fn with_flex_direction(mut self, flex_direction: taffy::FlexDirection) -> Self {
        self.flex_direction = flex_direction;
        self
    }

    pub fn with_flex_wrap(mut self, flex_wrap: taffy::FlexWrap) -> Self {
        self.flex_wrap = flex_wrap;
        self
    }

    pub fn with_flex_basis(mut self, flex_basis: taffy::Dimension) -> Self {
        self.flex_basis = flex_basis;
        self
    }

    pub fn with_flex_grow(mut self, flex_grow: f32) -> Self {
        self.flex_grow = flex_grow;
        self
    }

    pub fn with_flex_shrink(mut self, flex_shrink: f32) -> Self {
        self.flex_shrink = flex_shrink;
        self
    }

    // TODO: Expose grid stuff.
}

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
                taffy
                    .new_leaf_with_context(style.0.clone(), measure)
                    .unwrap()
            } else {
                taffy.new_leaf(style.0.clone()).unwrap()
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

            taffy.set_style(taffy_node_id, style.0.clone()).unwrap();
        }
    }

    pub fn add_children(&mut self, entity: Entity, children: &Children) {
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
        self.taffy
            .compute_layout(
                *root_id,
                // Size::max_content(),
                Size {
                    width: taffy::AvailableSpace::Definite(root_node_size.x),
                    height: taffy::AvailableSpace::Definite(root_node_size.y),
                },
            )
            .unwrap();
        // self.taffy.print_tree(*root_id);
        // self.taffy
        //     .compute_layout_with_measure(
        //         *root_id,
        //         Size {
        //             width: taffy::AvailableSpace::Definite(root_node_size.x),
        //             height: taffy::AvailableSpace::Definite(root_node_size.y),
        //         },
        //         |known_dimensions: taffy::Size<Option<f32>>,
        //          available_space: taffy::Size<taffy::AvailableSpace>,
        //          _node_id: taffy::NodeId,
        //          context: Option<&mut LayoutMeasure>,
        //          style: &taffy::Style|
        //          -> taffy::Size<f32> {
        //             context
        //                 .map(|ctx| {
        //                     let size = ctx.measure(
        //                         known_dimensions.width,
        //                         known_dimensions.height,
        //                         available_space.width,
        //                         available_space.height,
        //                         style,
        //                     );
        //                     taffy::Size {
        //                         width: size.x,
        //                         height: size.y,
        //                     }
        //                 })
        //                 .unwrap_or(taffy::Size::ZERO)
        //         },
        //     )
        //     .unwrap();
    }
}
