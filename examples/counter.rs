use bevy::prelude::*;
use woodpecker_ui::prelude::*;

#[derive(Component, PartialEq, Default, Debug, Clone)]
pub struct CounterState {
    count: u32,
}

#[derive(Widget, Component, Reflect, PartialEq, Default, Debug, Clone)]
#[auto_update(render)]
#[props(CounterWidget)]
#[state(CounterState)]
#[require(WoodpeckerStyle, WidgetChildren)]
pub struct CounterWidget {
    initial_count: u32,
}

fn render(
    current_widget: Res<CurrentWidget>,
    mut commands: Commands,
    mut query: Query<(&CounterWidget, &mut WidgetChildren)>,
    state_query: Query<&CounterState>,
    mut hooks: ResMut<HookHelper>,
) {
    let Ok((widget, mut children)) = query.get_mut(**current_widget) else {
        return;
    };

    let state_entity = hooks.use_state(
        &mut commands,
        *current_widget,
        CounterState {
            count: widget.initial_count,
        },
    );

    let Ok(state) = state_query.get(state_entity) else {
        return;
    };

    // Dereference so we don't move the reference into the on click closure.
    let current_widget = *current_widget;
    *children = WidgetChildren::default().with_child::<Element>((
        Element,
        WoodpeckerStyle {
            width: Units::Percentage(100.0),
            flex_direction: WidgetFlexDirection::Column,
            justify_content: Some(WidgetAlignContent::Center),
            align_items: Some(WidgetAlignItems::Center),
            ..Default::default()
        },
        WidgetChildren::default()
            .with_child::<Element>((
                Element,
                WoodpeckerStyle {
                    font_size: 50.0,
                    margin: Edge::all(10.0),
                    ..Default::default()
                },
                WidgetRender::Text {
                    content: format!("Current Count: {}", state.count),
                    word_wrap: false,
                },
            ))
            .with_child::<WButton>((
                WButton,
                WidgetChildren::default().with_child::<Element>((
                    Element,
                    WoodpeckerStyle {
                        font_size: 14.0,
                        margin: Edge::all(10.0),
                        ..Default::default()
                    },
                    WidgetRender::Text {
                        content: "Increase Count".into(),
                        word_wrap: false,
                    },
                )),
            ))
            .with_observe(
                current_widget,
                move |_: Trigger<Pointer<Click>>, mut query: Query<&mut CounterState>| {
                    let Ok(mut state) = query.get_mut(state_entity) else {
                        return;
                    };
                    state.count += 1;
                },
            ),
    ));

    children.apply(current_widget.as_parent());
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WoodpeckerUIPlugin::default())
        .add_systems(Startup, startup)
        .register_widget::<CounterWidget>()
        .run();
}

fn startup(
    mut commands: Commands,
    mut ui_context: ResMut<WoodpeckerContext>,
    mut font_manager: ResMut<FontManager>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn((Camera2d, WoodpeckerView));

    let font = asset_server.load("Outfit/static/Outfit-Regular.ttf");
    font_manager.add(&font);

    let root = commands
        .spawn((
            WoodpeckerApp,
            WidgetChildren::default().with_child::<CounterWidget>((
                CounterWidget { initial_count: 0 },
                WoodpeckerStyle {
                    width: Units::Percentage(100.0),
                    ..Default::default()
                },
            )),
        ))
        .id();
    ui_context.set_root_widget(root);
}
