use std::sync::Arc;

use bevy::prelude::*;

use crate::{context::Widget, prelude::WidgetMapper, ParentWidget};

/// A commponent to pass children down the tree
/// while also having children of its own.
#[derive(Component, Default, Clone, Deref, DerefMut, PartialEq)]
pub struct PassedChildren(pub WidgetChildren);

/// A commponent to pass children down the tree
/// while also having children of its own.
#[derive(Component, Default, Clone)]
pub struct Mounted;

/// A bevy component that keeps track of Woodpecker UI widget children.
/// This is very similar to bevy commands as in it lets you spawn bundles
/// but it does not create an entity until
/// [`WidgetChildren::process`] is called.
#[derive(Component, Default, Clone)]
pub struct WidgetChildren {
    // Strings here are widget type names.
    // First children are stored in a queue.
    children_queue: Vec<(
        String,
        Arc<dyn Fn(&mut World, &mut WidgetMapper, ParentWidget, usize, String) + Sync + Send>,
    )>,
    // When a widget is processed onto a parent they get stored here and removed from the queue.
    children: Vec<(
        String,
        Arc<dyn Fn(&mut World, &mut WidgetMapper, ParentWidget, usize, String) + Sync + Send>,
    )>,
    /// Stores a list of previous children.
    prev_children: Vec<String>,
    /// Lets the system know who the parent is.
    /// We need this because childen can be passed around until they
    /// are committed to a parent.
    parent_widget: Option<ParentWidget>,
}

impl PartialEq for WidgetChildren {
    fn eq(&self, other: &Self) -> bool {
        let queue = self
            .children_queue
            .iter()
            .map(|(wn, _)| wn.clone())
            .collect::<Vec<_>>();
        let other_queue = other
            .children_queue
            .iter()
            .map(|(wn, _)| wn.clone())
            .collect::<Vec<_>>();
        let children: Vec<String> = self
            .children
            .iter()
            .map(|(wn, _)| wn.clone())
            .collect::<Vec<_>>();
        let other_children = other
            .children
            .iter()
            .map(|(wn, _)| wn.clone())
            .collect::<Vec<_>>();
        queue == other_queue && children == other_children
    }
}

impl WidgetChildren {
    /// Builder pattern for adding children when you initially create the component.
    pub fn with_child<T: Widget>(mut self, bundle: impl Bundle + Clone) -> Self {
        self.add::<T>(bundle);
        self
    }

    /// Clears out all children.
    pub fn clear(&mut self) {
        self.children.clear();
        self.parent_widget = None;
    }

    /// Add a new widget to the list of children. The widget should be a bevy bundle and T should implement widget.
    ///
    /// Note: Make sure to call [`WidgetChildren::process`] in the render system of the parent
    /// otherwise the entities will not be spawned! This will NOT spawn the bundles.
    pub fn add<T: Widget>(&mut self, bundle: impl Bundle + Clone) -> &mut Self {
        let widget_type = T::get_name();
        self.children_queue.push((
            widget_type,
            Arc::new(
                move |world: &mut World,
                widget_mapper: &mut WidgetMapper,
                parent: ParentWidget,
                index: usize, widget_type: String| {
                    let type_name_without_path = widget_type.clone().split("::").last().unwrap().to_string();
                    let child_widget = widget_mapper.get_or_insert_entity_world(
                        world,
                        widget_type,
                        parent,
                        None,
                        index,
                    );
                    world
                        .entity_mut(child_widget)
                        .insert(bundle.clone())
                        .insert(Mounted)
                        .insert(Name::new(type_name_without_path.clone()));
                },
            ),
        ));

        self
    }

    /// Lets you know if the children have changed between now and when they were last rendered.
    pub fn children_changed(&self) -> bool {
        self.children.iter().map(|(n, _)| n).collect::<Vec<_>>()
            != self.prev_children.iter().collect::<Vec<_>>()
    }

    /// Attaches the children to a parent widget.
    /// Note: This doesn't actually spawn the children
    /// that occurs when the parent widget finishes rendering.
    pub fn apply(&mut self, parent_widget: ParentWidget) {
        self.parent_widget = Some(parent_widget);
    }

    pub(crate) fn process_world(&mut self, world: &mut World) {
        let Some(parent_widget) = self.parent_widget else {
            return;
        };

        // If our queue isn't empty drain it into the children
        // This ensures we remove previous children.
        if !self.children_queue.is_empty() {
            self.children = self.children_queue.drain(..).collect::<Vec<_>>();
        }

        world.resource_scope(|world: &mut World, mut widget_mapper: Mut<WidgetMapper>| {
            // Loop through each child and spawn the bundles.
            // The widget mapper helps keep track of which entities go with which child.
            // They are ensured to have the same entity id for a given child index and
            // widget type name. The type name is passed in here from the children vec.
            for (i, (widget_type, child)) in self.children.iter().enumerate() {
                trace!("Adding as child: {}", widget_type);
                child(world, &mut widget_mapper, parent_widget, i, widget_type.clone());
            }
        });

        // Remove the parent widget.
        // [`Self::apply`] should be called if this parent re-renders.
        // If its not then we assume no children.
        self.parent_widget = None;

        // Throw the children type names into the previous children list.
        self.prev_children = self
            .children
            .iter()
            .map(|(n, _)| n.clone())
            .collect::<Vec<_>>()
    }
}
