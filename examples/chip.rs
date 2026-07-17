use bevy::prelude::*;
use woodpecker_ui::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WoodpeckerUIPlugin::default())
        .register_widget::<ChipDemo>()
        .add_systems(Startup, startup)
        .run();
}

const FILTERS: [&str; 3] = ["Design", "Backend", "Urgent"];

#[derive(Component, Default, Clone, PartialEq, Reflect)]
#[reflect(Component, DiffableProp, PartialEq)]
struct ChipDemoState {
    /// One entry per `FILTERS` -- whether that filter chip is currently selected.
    filters_selected: Vec<bool>,
    /// Deletable tag chips, removed from this list on `ChipDeleted`.
    tags: Vec<String>,
}

impl ChipDemoState {
    fn initial() -> Self {
        Self {
            filters_selected: vec![true, false, false],
            tags: vec!["design".into(), "urgent".into(), "backend".into()],
        }
    }
}

fn section_label(text: &str) -> impl Bundle + Clone {
    (
        Element,
        WoodpeckerStyle {
            font_size: 13.0,
            color: Srgba::new(0.72, 0.75, 0.8, 1.0).into(),
            margin: Edge::all(0.0).bottom(8.0),
            ..Default::default()
        },
        WidgetRender::Text {
            content: text.into(),
        },
    )
}

#[derive(Widget, Component, Default, Clone, PartialEq, Reflect)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(render)]
#[require(WoodpeckerStyle, WidgetChildren)]
struct ChipDemo;

fn render(
    mut commands: Commands,
    current_widget: Res<CurrentWidget>,
    mut hooks: ResMut<HookHelper>,
    mut query: Query<&mut WidgetChildren, With<ChipDemo>>,
    state_query: Query<&ChipDemoState>,
) {
    let Ok(mut children) = query.get_mut(**current_widget) else {
        return;
    };

    let state_entity = hooks.use_state(&mut commands, *current_widget, ChipDemoState::initial());
    let Ok(state) = state_query.get(state_entity) else {
        return;
    };
    let state = state.clone();
    let current_widget = *current_widget;

    *children = WidgetChildren::default();

    children.add::<Element>((
        Element,
        WoodpeckerStyle {
            width: Units::Percentage(100.0),
            flex_direction: WidgetFlexDirection::Column,
            margin: Edge::all(0.0).bottom(20.0),
            ..Default::default()
        },
        WidgetChildren::default()
            .with_child::<Element>(section_label("Filter by (click to toggle)"))
            .with_key("label")
            .with_child::<Element>((
                Element,
                WoodpeckerStyle {
                    flex_direction: WidgetFlexDirection::Row,
                    gap: (8.0.into(), 0.0.into()),
                    ..Default::default()
                },
                {
                    let mut row = WidgetChildren::default();
                    for (index, label) in FILTERS.iter().enumerate() {
                        row.add::<Chip>((Chip {
                            label: (*label).into(),
                            variant: BadgeVariant::Info,
                            selected: state.filters_selected[index],
                            ..Default::default()
                        },))
                            .observe(
                                current_widget,
                                move |_trigger: On<Change<ChipClicked>>,
                                      mut query: Query<&mut ChipDemoState>| {
                                    if let Ok(mut state) = query.get_mut(state_entity) {
                                        state.filters_selected[index] =
                                            !state.filters_selected[index];
                                    }
                                },
                            );
                        row.add_key(*label);
                    }
                    row
                },
            ))
            .with_key("row"),
    ));
    children.add_key("filters");

    children.add::<Element>((
        Element,
        WoodpeckerStyle {
            width: Units::Percentage(100.0),
            flex_direction: WidgetFlexDirection::Column,
            ..Default::default()
        },
        WidgetChildren::default()
            .with_child::<Element>(section_label("Tags (click \u{2715} to remove)"))
            .with_key("label")
            .with_child::<Element>((
                Element,
                WoodpeckerStyle {
                    flex_direction: WidgetFlexDirection::Row,
                    gap: (8.0.into(), 0.0.into()),
                    ..Default::default()
                },
                {
                    let mut row = WidgetChildren::default();
                    for tag in &state.tags {
                        let tag_key = tag.clone();
                        row.add::<Chip>((Chip {
                            label: tag.clone(),
                            deletable: true,
                            ..Default::default()
                        },))
                            .observe(
                                current_widget,
                                move |_trigger: On<Change<ChipDeleted>>,
                                      mut query: Query<&mut ChipDemoState>| {
                                    if let Ok(mut state) = query.get_mut(state_entity) {
                                        state.tags.retain(|t| t != &tag_key);
                                    }
                                },
                            );
                        row.add_key(tag.clone());
                    }
                    row
                },
            ))
            .with_key("row"),
    ));
    children.add_key("tags");

    children.apply(current_widget.as_parent());
}

fn startup(mut commands: Commands, mut ui_context: ResMut<WoodpeckerContext>) {
    commands.spawn((Camera2d, WoodpeckerView));

    let root_widget = ui_context.spawn_root(&mut commands);
    commands.entity(*root_widget).insert((
        WoodpeckerStyle {
            padding: Edge::all(20.0),
            width: 420.0.into(),
            ..Default::default()
        },
        WidgetChildren::default()
            .with_child::<ChipDemo>(ChipDemo)
            .with_key("demo"),
    ));
}
