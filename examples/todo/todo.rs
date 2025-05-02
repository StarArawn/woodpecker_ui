use bevy::prelude::*;
use bevy_vello::render::VelloView;
use woodpecker_ui::prelude::*;

mod input;
mod list;

use input::{TodoInput, TodoInputBundle};
use list::{TodoList, TodoListBundle};

#[derive(Resource, Deref, DerefMut)]
pub struct TodoListData(Vec<String>);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WoodpeckerUIPlugin::default())
        .register_widget::<TodoList>()
        .register_widget::<TodoInput>()
        .insert_resource(TodoListData(vec![
            "Walk the dog!".into(),
            "Walk the cat!".into(),
            "Walk the human?".into(),
            "Cleanup the house.".into(),
            "Build a new UI library...".into(),
            "Go to the gym.".into(),
            "Write a book.".into(),
            "Learn a new skill.".into(),
            "Get the dream job.".into(),
        ]))
        .add_systems(Startup, startup)
        .run();
}

fn startup(mut commands: Commands, mut ui_context: ResMut<WoodpeckerContext>) {
    commands.spawn((Camera2d, VelloView));

    let root = commands
        .spawn(WoodpeckerAppBundle {
            children: WidgetChildren::default().with_child::<Modal>(ModalBundle {
                modal: Modal {
                    visible: true,
                    title: "Todo Example".into(),
                    min_size: Vec2::new(500.0, 350.0),
                    ..Default::default()
                },
                children: PassedChildren(
                    WidgetChildren::default().with_child::<ScrollContextProvider>((
                        ScrollContextProviderBundle {
                            styles: WoodpeckerStyle {
                                width: Units::Percentage(100.0),
                                height: Units::Percentage(100.0),
                                ..Default::default()
                            },
                            children: WidgetChildren::default().with_child::<ScrollBox>(
                                ScrollBoxBundle {
                                    children: PassedChildren(
                                        WidgetChildren::default().with_child::<Element>((
                                            ElementBundle {
                                                styles: WoodpeckerStyle {
                                                    padding: Edge::all(0.0).left(10.0).right(10.0),
                                                    flex_direction: WidgetFlexDirection::Column,
                                                    ..Default::default()
                                                },
                                                children: WidgetChildren::default()
                                                    .with_child::<TodoInput>(TodoInputBundle {
                                                        ..Default::default()
                                                    })
                                                    .with_child::<TodoList>(
                                                        TodoListBundle::default(),
                                                    ),
                                                ..Default::default()
                                            },
                                        )),
                                    ),
                                    ..Default::default()
                                },
                            ),
                            ..Default::default()
                        },
                    )),
                ),
                ..Default::default()
            }),
            ..Default::default()
        })
        .id();
    ui_context.set_root_widget(root);
}
