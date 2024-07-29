use bevy::prelude::*;
use bevy_mod_picking::{
    events::{Click, Pointer},
    prelude::{Listener, On},
};
use woodpecker_ui::prelude::*;

use crate::TodoListData;

#[derive(Widget, Component, Reflect, Clone, Default)]
#[widget_systems(update, render)]
pub struct TodoInput {
    current_value: String,
}

#[derive(Bundle, Clone)]
pub struct TodoInputBundle {
    pub todo: TodoInput,
    pub children: WidgetChildren,
    pub styles: WoodpeckerStyle,
}

impl Default for TodoInputBundle {
    fn default() -> Self {
        Self {
            todo: TodoInput {
                current_value: "".into(),
            },
            children: Default::default(),
            styles: WoodpeckerStyle {
                width: Units::Percentage(100.0),
                justify_content: Some(WidgetJustifyContent::SpaceBetween),
                margin: Edge::new(10.0, 0.0, 15.0, 0.0),
                ..Default::default()
            },
        }
    }
}

fn update(
    current_widget: Res<CurrentWidget>,
    todo_list_data: Res<TodoListData>,
    query: Query<Entity, Added<TodoInput>>,
) -> bool {
    todo_list_data.is_changed() || query.contains(**current_widget)
}

fn render(current_widget: Res<CurrentWidget>, mut query: Query<(&TodoInput, &mut WidgetChildren)>) {
    let Ok((input, mut children)) = query.get_mut(**current_widget) else {
        return;
    };

    // Dereference so we can copy into the closure.
    let current_widget = *current_widget;
    children
        .add::<TextBox>((
            TextBoxBundle {
                text_box: TextBox {
                    initial_value: input.current_value.clone(),
                    ..Default::default()
                },
                textbox_styles: TextboxStyles {
                    normal: WoodpeckerStyle {
                        width: Units::Pixels(400.0),
                        ..TextboxStyles::default().normal
                    },
                    hovered: WoodpeckerStyle {
                        width: Units::Pixels(400.0),
                        ..TextboxStyles::default().hovered
                    },
                    focused: WoodpeckerStyle {
                        width: Units::Pixels(400.0),
                        ..TextboxStyles::default().focused
                    },
                },
                ..Default::default()
            },
            On::<OnChange<ChangedText>>::run(
                move |event: Listener<OnChange<ChangedText>>, mut query: Query<&mut TodoInput>| {
                    let Ok(mut input) = query.get_mut(*current_widget) else {
                        return;
                    };
                    input.current_value = event.data.value.clone();
                },
            ),
        ))
        .add::<WButton>((
            WButtonBundle {
                button_styles: ButtonStyles {
                    normal: WoodpeckerStyle {
                        margin: Edge::all(0.0).left(10.0),
                        width: 100.0.into(),
                        ..ButtonStyles::default().normal
                    },
                    hovered: WoodpeckerStyle {
                        margin: Edge::all(0.0).left(10.0),
                        width: 100.0.into(),
                        ..ButtonStyles::default().hovered
                    },
                },
                children: WidgetChildren::default().with_child::<Element>((
                    ElementBundle {
                        styles: WoodpeckerStyle {
                            font_size: 14.0,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    WidgetRender::Text {
                        content: "Add".into(),
                        word_wrap: true,
                    },
                )),
                ..Default::default()
            },
            On::<Pointer<Click>>::run(
                move |mut data: ResMut<TodoListData>, mut query: Query<&mut TodoInput>| {
                    let Ok(mut input) = query.get_mut(*current_widget) else {
                        return;
                    };

                    if input.current_value.trim().is_empty() {
                        return;
                    }

                    data.insert(0, input.current_value.clone());
                    input.current_value.clear();
                },
            ),
        ));

    children.apply(current_widget.as_parent());
}
