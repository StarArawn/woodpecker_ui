use std::sync::Arc;

use bevy::prelude::*;

use crate::{context::Widget, prelude::WidgetMapper, CurrentWidget, ParentWidget};

/// A bevy component that keeps track of Woodpecker UI widget children.
/// This is very similar to bevy commands as in it lets you spawn bundles
/// but it does not create an entity until
/// [`WidgetChildren::process`] is called.
#[derive(Component, Default, Clone)]
pub struct WidgetChildren {
    children: Vec<(
        String,
        Arc<dyn Fn(&mut World, &mut WidgetMapper, ParentWidget, usize) + Sync + Send>,
    )>,
    parent_widget: Option<ParentWidget>,
}

impl WidgetChildren {
    /// Add a new widget to the list of children. The widget should be a bevy bundle and T should implement widget.
    ///
    /// Note: Make sure to call [`WidgetChildren::process`] in the render system of the parent
    /// otherwise the entities will not be spawned!
    pub fn add<T: Widget>(&mut self, bundle: impl Bundle + Clone) {
        let widget_name = T::get_name();
        self.children.push((
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
    }

    pub fn process(&mut self, parent_widget: ParentWidget) {
        self.parent_widget = Some(parent_widget);
    }

    pub(crate) fn process_world(&mut self, world: &mut World) {
        let Some(parent_widget) = self.parent_widget else {
            return;
        };
        world.resource_scope(|world: &mut World, mut widget_mapper: Mut<WidgetMapper>| {
            for (i, (_, child)) in self.children.iter().enumerate() {
                child(world, &mut widget_mapper, parent_widget, i);
            }
        });
        world.remove_resource::<CurrentWidget>();
        self.parent_widget = None;
    }
}
