use bevy::prelude::*;
use bevy_mod_picking::{
    events::{Click, Pointer},
    prelude::On,
};
use woodpecker_ui::prelude::*;

use crate::TodoListData;

#[derive(Widget, Component, Reflect, Clone, Default)]
#[widget_systems(update, render)]
pub struct TodoList;

#[derive(Bundle, Clone)]
pub struct TodoListBundle {
    pub todo: TodoList,
    pub children: WidgetChildren,
    pub styles: WoodpeckerStyle,
}

impl Default for TodoListBundle {
    fn default() -> Self {
        Self {
            todo: TodoList,
            children: Default::default(),
            styles: WoodpeckerStyle {
                width: Units::Percentage(100.0),
                ..Default::default()
            },
        }
    }
}

fn update(
    current_widget: Res<CurrentWidget>,
    todo_list_data: Res<TodoListData>,
    query: Query<Entity, Added<TodoList>>,
) -> bool {
    todo_list_data.is_changed() || query.contains(**current_widget)
}

fn render(
    current_widget: Res<CurrentWidget>,
    todo_list_data: Res<TodoListData>,
    mut query: Query<&mut WidgetChildren>,
) {
    let Ok(mut children) = query.get_mut(**current_widget) else {
        return;
    };

    let mut todo_children = WidgetChildren::default();

    for (i, todo) in todo_list_data.iter().enumerate() {
        todo_children.add::<Element>((
            ElementBundle {
                styles: WoodpeckerStyle {
                    background_color: colors::DARK_BACKGROUND,
                    width: Units::Percentage(100.0),
                    margin: Edge::all(5.0).bottom(15.0),
                    padding: Edge::all(15.0),
                    border: Edge::all(2.0),
                    border_color: colors::PRIMARY,
                    border_radius: Corner::all(5.0),
                    align_items: Some(WidgetAlignItems::Center),
                    justify_content: Some(WidgetJustifyContent::SpaceBetween),
                    ..Default::default()
                },
                children: WidgetChildren::default()
                    .with_child::<Element>((
                        ElementBundle {
                            styles: WoodpeckerStyle {
                                font_size: 14.0,
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        WidgetRender::Text {
                            content: todo.clone(),
                            word_wrap: true,
                        },
                    ))
                    .with_child::<WButton>((
                        WButtonBundle {
                            button_styles: ButtonStyles {
                                normal: WoodpeckerStyle {
                                    margin: Edge::new(0.0, 0.0, 0.0, 0.0),
                                    width: 100.0.into(),
                                    ..ButtonStyles::default().normal
                                },
                                hovered: WoodpeckerStyle {
                                    margin: Edge::new(0.0, 0.0, 0.0, 0.0),
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
                                    content: "Done".into(),
                                    word_wrap: true,
                                },
                            )),
                            ..Default::default()
                        },
                        On::<Pointer<Click>>::run(
                            move |mut todo_list_data: ResMut<TodoListData>| {
                                todo_list_data.remove(i);
                            },
                        ),
                    )),
                ..Default::default()
            },
            WidgetRender::Quad,
        ));
    }

    children.add::<Element>(ElementBundle {
        styles: WoodpeckerStyle {
            flex_direction: WidgetFlexDirection::Column,
            width: Units::Percentage(100.0),
            height: Units::Auto,
            margin: Edge::new(0.0, 0.0, 50.0, 0.0),
            ..Default::default()
        },
        children: todo_children,
        ..Default::default()
    });

    children.apply(current_widget.as_parent());
}
