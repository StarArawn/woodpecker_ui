use std::sync::Arc;

use bevy::prelude::*;

use crate::{context::Widget, prelude::WidgetMapper, CurrentWidget, ParentWidget};

/// A commponent to pass children down the tree
/// while also having children of its own.
#[derive(Component, Default, Clone, Deref, DerefMut)]
pub struct PassedChildren(pub WidgetChildren);

/// A bevy component that keeps track of Woodpecker UI widget children.
/// This is very similar to bevy commands as in it lets you spawn bundles
/// but it does not create an entity until
/// [`WidgetChildren::process`] is called.
#[derive(Component, Default, Clone)]
pub struct WidgetChildren {
    prev_children: Vec<String>,
    children_queue: Vec<(
        String,
        Arc<dyn Fn(&mut World, &mut WidgetMapper, ParentWidget, usize) + Sync + Send>,
    )>,
    children: Vec<(
        String,
        Arc<dyn Fn(&mut World, &mut WidgetMapper, ParentWidget, usize) + Sync + Send>,
    )>,
    parent_widget: Option<ParentWidget>,
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
        let widget_name = T::get_name();
        self.children_queue.push((
            T::get_name(),
            Arc::new(
                move |world: &mut World,
                      widget_mapper: &mut WidgetMapper,
                      parent: ParentWidget,
                      index: usize| {
                    let child_widget = widget_mapper.get_or_insert_entity_world(
                        world,
                        widget_name.clone(),
                        parent,
                        None,
                        index,
                    );
                    world.entity_mut(child_widget).insert(bundle.clone());
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

        if !self.children_queue.is_empty() {
            self.children = self.children_queue.drain(..).collect::<Vec<_>>();
        }

        world.resource_scope(|world: &mut World, mut widget_mapper: Mut<WidgetMapper>| {
            for (i, (_, child)) in self.children.iter().enumerate() {
                child(world, &mut widget_mapper, parent_widget, i);
            }
        });
        world.remove_resource::<CurrentWidget>();
        self.parent_widget = None;
        self.prev_children = self
            .children
            .iter()
            .map(|(n, _)| n.clone())
            .collect::<Vec<_>>()
    }
}
