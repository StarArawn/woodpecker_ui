use crate::prelude::*;
use bevy::prelude::*;
// use bevy_mod_picking::{
//     events::{Click, Pointer},
//     prelude::{ListenerMut, On, Pickable, PointerInteraction},
// };

/// A textbox change event.
#[derive(Debug, Clone, Reflect)]
pub struct DropdownChanged {
    /// The current text value
    pub value: String,
}

/// Dropdown Styles
#[derive(Component, Clone, PartialEq, Reflect)]
pub struct DropdownStyles {
    /// Dropdown Background Styles
    pub background: WoodpeckerStyle,
    /// Dropdown Text Styles
    pub text: WoodpeckerStyle,
    /// Dropdown Icon Styles
    pub icon: WoodpeckerStyle,
    /// Dropdown List Area Styles
    pub list_area: WoodpeckerStyle,
    /// Dropdown List Item Styles
    pub list_item: ButtonStyles,
}

impl Default for DropdownStyles {
    fn default() -> Self {
        Self {
            background: WoodpeckerStyle {
                align_items: Some(WidgetAlignItems::Center),
                background_color: colors::BACKGROUND,
                width: Units::Percentage(100.0),
                height: 54.0.into(),
                padding: Edge::all(0.0).left(30.0).right(10.0),
                ..Default::default()
            },
            text: WoodpeckerStyle {
                color: Color::WHITE,
                font_size: 32.0,
                flex_grow: 1.0,
                ..Default::default()
            },
            icon: WoodpeckerStyle {
                color: Color::WHITE,
                width: 32.0.into(),
                height: 32.0.into(),
                ..Default::default()
            },
            list_area: WoodpeckerStyle {
                background_color: colors::BACKGROUND,
                position: WidgetPosition::Absolute,
                left: 0.0.into(), // 0px because padding is 30px.
                top: (54.0 + 20.0).into(),
                width: Units::Percentage(100.0),
                min_height: 54.0.into(),
                flex_direction: WidgetFlexDirection::Column,
                ..Default::default()
            },
            list_item: ButtonStyles {
                normal: WoodpeckerStyle {
                    align_items: Some(WidgetAlignItems::Center),
                    background_color: colors::BACKGROUND,
                    width: Units::Percentage(100.0),
                    height: 54.0.into(),
                    padding: Edge::all(0.0).left(30.0).right(10.0),
                    ..Default::default()
                },
                hovered: WoodpeckerStyle {
                    align_items: Some(WidgetAlignItems::Center),
                    background_color: colors::BACKGROUND_LIGHT,
                    width: Units::Percentage(100.0),
                    height: 54.0.into(),
                    padding: Edge::all(0.0).left(30.0).right(10.0),
                    ..Default::default()
                },
            },
        }
    }
}

/// Dropdown state
#[derive(Default, Debug, Component, Clone, PartialEq, Reflect)]
pub struct DropdownState {
    /// Is open?
    is_open: bool,
    /// The current value
    current_value: String,
}

/// A dropdown widget
#[derive(Widget, Default, Component, Clone, PartialEq, Reflect)]
#[auto_update(render)]
#[props(Dropdown)]
#[state(DropdownState)]
pub struct Dropdown {
    /// The current value
    pub current_value: String,
    /// A list of items in the dropdown
    pub list: Vec<String>,
    /// Styles
    pub styles: DropdownStyles,
}

/// A bundle for convince when creating the widget.
#[derive(Bundle, Clone)]
pub struct DropdownBundle {
    /// The dropdown component
    pub dropdown: Dropdown,
    /// The widget style component
    pub styles: WoodpeckerStyle,
    /// An internal widget render
    pub widget_render: WidgetRender,
    /// Internal children
    pub children: WidgetChildren,
    /// Picking
    pub pickable: Pickable,
    /// Can focus
    pub focusable: Focusable,
}

impl Default for DropdownBundle {
    fn default() -> Self {
        Self {
            dropdown: Default::default(),
            styles: Default::default(),
            widget_render: WidgetRender::Quad,
            children: Default::default(),
            pickable: Default::default(),
            focusable: Focusable,
        }
    }
}

fn render(
    mut commands: Commands,
    mut hooks: ResMut<HookHelper>,
    current_widget: Res<CurrentWidget>,
    asset_server: Res<AssetServer>,
    mut query: Query<(&Dropdown, &mut WoodpeckerStyle, &mut WidgetChildren)>,
    state_query: Query<&DropdownState>,
) {
    let Ok((dropdown, mut styles, mut children)) = query.get_mut(**current_widget) else {
        return;
    };

    let state_entity = hooks.use_state(
        &mut commands,
        *current_widget,
        DropdownState {
            is_open: false,
            current_value: dropdown.current_value.clone(),
        },
    );

    let Ok(state) = state_query.get(state_entity) else {
        return;
    };

    *styles = dropdown.styles.background;

    *children = WidgetChildren::default()
        .with_observe(
            *current_widget,
            move |_trigger: Trigger<Pointer<Click>>, mut state_query: Query<&mut DropdownState>| {
                let Ok(mut state) = state_query.get_mut(state_entity) else {
                    return;
                };

                state.is_open = !state.is_open;
            },
        )
        // Text
        .with_child::<Element>((
            ElementBundle {
                styles: dropdown.styles.text,
                ..Default::default()
            },
            WidgetRender::Text {
                content: state.current_value.clone(),
                word_wrap: false,
            },
        ))
        // Icon
        .with_child::<Element>((
            ElementBundle {
                styles: dropdown.styles.icon,
                ..Default::default()
            },
            WidgetRender::Svg {
                handle: if state.is_open {
                    asset_server.load("embedded://woodpecker_ui/embedded_assets/icons/arrow-up.svg")
                } else {
                    asset_server
                        .load("embedded://woodpecker_ui/embedded_assets/icons/arrow-down.svg")
                },
                color: Some(dropdown.styles.icon.color),
            },
        ));

    let dropdown_entity = **current_widget;
    let mut list_children = WidgetChildren::default();

    // List area
    for (i, item) in dropdown.list.iter().enumerate() {
        list_children
            .add::<WButton>((WButtonBundle {
                children: WidgetChildren::default().with_child::<Element>((
                    ElementBundle {
                        styles: dropdown.styles.text,
                        ..Default::default()
                    },
                    WidgetRender::Text {
                        content: item.clone(),
                        word_wrap: false,
                    },
                )),
                button_styles: dropdown.styles.list_item,
                ..Default::default()
            },))
            .observe(
                *current_widget,
                move |mut trigger: Trigger<Pointer<Click>>,
                      mut commands: Commands,
                      mut state_query: Query<&mut DropdownState>,
                      dropdown_query: Query<&Dropdown>| {
                    trigger.propagate(false);
                    let Ok(mut state) = state_query.get_mut(state_entity) else {
                        return;
                    };
                    let Ok(dropdown) = dropdown_query.get(dropdown_entity) else {
                        return;
                    };
                    state.current_value.clone_from(&dropdown.list[i]);
                    state.is_open = false;
                    commands.trigger_targets(
                        Change {
                            target: dropdown_entity,
                            data: DropdownChanged {
                                value: state.current_value.clone(),
                            },
                        },
                        dropdown_entity,
                    );
                },
            );
    }
    children.add::<Element>((
        ElementBundle {
            styles: WoodpeckerStyle {
                display: if state.is_open {
                    WidgetDisplay::Flex
                } else {
                    WidgetDisplay::None
                },
                ..dropdown.styles.list_area
            },
            children: list_children,
            ..Default::default()
        },
        WidgetRender::Quad,
    ));

    children.apply(current_widget.as_parent());
}
