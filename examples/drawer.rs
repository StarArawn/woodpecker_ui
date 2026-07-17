use bevy::prelude::*;
use woodpecker_ui::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WoodpeckerUIPlugin::default())
        .register_widget::<DrawerDemo>()
        .add_systems(Startup, startup)
        .run();
}

#[derive(Component, Default, Clone, Copy, PartialEq, Reflect)]
#[reflect(Component, DiffableProp, PartialEq)]
struct DrawerDemoState {
    open: bool,
}

#[derive(Widget, Component, Default, Clone, PartialEq, Reflect)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(render)]
#[require(WoodpeckerStyle, WidgetChildren)]
struct DrawerDemo;

fn nav_item(label: &str) -> ListItem {
    ListItem {
        primary_text: label.into(),
        ..Default::default()
    }
}

fn render(
    mut commands: Commands,
    current_widget: Res<CurrentWidget>,
    mut hooks: ResMut<HookHelper>,
    mut query: Query<&mut WidgetChildren, With<DrawerDemo>>,
    state_query: Query<&DrawerDemoState>,
) {
    let Ok(mut children) = query.get_mut(**current_widget) else {
        return;
    };

    let state_entity = hooks.use_state(&mut commands, *current_widget, DrawerDemoState::default());
    let Ok(state) = state_query.get(state_entity) else {
        return;
    };
    let open = state.open;

    *children = WidgetChildren::default();
    children.add::<WButton>(WButton::text("Open menu")).observe(
        *current_widget,
        move |_trigger: On<Pointer<Click>>, mut query: Query<&mut DrawerDemoState>| {
            if let Ok(mut state) = query.get_mut(state_entity) {
                state.open = true;
            }
        },
    );
    children.add_key("open");

    children.add::<Drawer>((
        Drawer {
            open,
            position: DrawerPosition::Left,
            ..Default::default()
        },
        PassedChildren(
            WidgetChildren::default().with_child::<List>((
                List,
                WoodpeckerStyle {
                    padding: Edge::all(20.0),
                    ..Default::default()
                },
                PassedChildren(
                    WidgetChildren::default()
                        .with_child::<ListItem>(nav_item("Dashboard"))
                        .with_key("dashboard")
                        .with_child::<ListItem>(nav_item("Reports"))
                        .with_key("reports")
                        .with_child::<ListItem>(nav_item("Settings"))
                        .with_key("settings"),
                ),
            )),
        ),
    ));
    children.add_key("drawer");
    children.observe(
        *current_widget,
        move |_trigger: On<Change<DrawerCloseRequested>>,
              mut query: Query<&mut DrawerDemoState>| {
            if let Ok(mut state) = query.get_mut(state_entity) {
                state.open = false;
            }
        },
    );

    children.apply(current_widget.as_parent());
}

fn startup(mut commands: Commands, mut ui_context: ResMut<WoodpeckerContext>) {
    commands.spawn((Camera2d, WoodpeckerView));

    let root_widget = ui_context.spawn_root(&mut commands);
    commands.entity(*root_widget).insert((
        WoodpeckerStyle {
            padding: Edge::all(20.0),
            ..Default::default()
        },
        WidgetChildren::default()
            .with_child::<DrawerDemo>(DrawerDemo)
            .with_key("demo")
            .with_child::<OverlayRootWidget>(OverlayRootWidget),
    ));
}
