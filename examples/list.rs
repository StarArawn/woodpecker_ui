use bevy::prelude::*;
use woodpecker_ui::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WoodpeckerUIPlugin::default())
        .add_systems(Startup, startup)
        .run();
}

struct ActivityEntry {
    key: &'static str,
    actor: &'static str,
    action: &'static str,
    time_ago: &'static str,
}

const ACTIVITY: [ActivityEntry; 4] = [
    ActivityEntry {
        key: "1",
        actor: "Ava Thompson",
        action: "deployed billing-service",
        time_ago: "2m ago",
    },
    ActivityEntry {
        key: "2",
        actor: "Marcus Chen",
        action: "merged auth-middleware",
        time_ago: "18m ago",
    },
    ActivityEntry {
        key: "3",
        actor: "System",
        action: "restarted worker-pool",
        time_ago: "1h ago",
    },
    ActivityEntry {
        key: "4",
        actor: "Priya Patel",
        action: "commented on #482",
        time_ago: "3h ago",
    },
];

fn time_label(content: &str) -> WidgetChildren {
    WidgetChildren::default().with_child::<Element>((
        Element,
        WoodpeckerStyle {
            font_size: 12.0,
            color: Srgba::new(0.6, 0.63, 0.7, 1.0).into(),
            text_wrap: TextWrap::None,
            ..Default::default()
        },
        WidgetRender::Text {
            content: content.into(),
        },
    ))
}

fn startup(mut commands: Commands, mut ui_context: ResMut<WoodpeckerContext>) {
    commands.spawn((Camera2d, WoodpeckerView));

    let mut rows = WidgetChildren::default();
    for entry in ACTIVITY {
        rows.add::<ListItem>((
            ListItem {
                primary_text: format!("{} {}", entry.actor, entry.action),
                selected: entry.key == "1",
                ..Default::default()
            },
            ListItemTrailing(time_label(entry.time_ago)),
        ));
        rows.add_key(entry.key);
    }

    let root_widget = ui_context.spawn_root(&mut commands);
    commands.entity(*root_widget).insert((
        WoodpeckerStyle {
            padding: Edge::all(20.0),
            width: 420.0.into(),
            ..Default::default()
        },
        WidgetChildren::default()
            .with_child::<List>((List, PassedChildren(rows)))
            .with_key("activity")
            .with_observe(root_widget, |trigger: On<Change<ListItemClicked>>| {
                info!("List item {:?} clicked", trigger.event_target());
            }),
    ));
}
