use crate::prelude::*;
use bevy::prelude::*;

/// A generic element widget used for layouts.
#[derive(Bundle, Default, Clone)]
pub struct ElementBundle {
    /// The element component itself.
    pub element: Element,
    /// A widget children component
    pub children: WidgetChildren,
    /// The widget styles,
    pub styles: WoodpeckerStyle,
}

impl ElementBundle {
    /// A convince function that lets you add children.
    pub fn with_child<T: Widget>(mut self, bundle: impl Bundle + Clone) -> Self {
        self.children.add::<T>(bundle);

        self
    }

    /// A convince function that lets you set the styles.
    pub fn with_style(mut self, style: WoodpeckerStyle) -> Self {
        self.styles = style;
        self
    }
}

/// The Woodpecker UI Element
#[derive(Component, Widget, PartialEq, Reflect, Default, Clone)]
#[auto_update(render)]
#[props(Element, WoodpeckerStyle)]
pub struct Element {}

pub fn render(entity: Res<CurrentWidget>, mut query: Query<&mut WidgetChildren>) {
    let Ok(mut children) = query.get_mut(**entity) else {
        return;
    };

    children.apply(entity.as_parent());
}
