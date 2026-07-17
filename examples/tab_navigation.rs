//! Manual test bed for Tab/Shift+Tab keyboard focus navigation and modal focus trapping.
//!
//! Try:
//! - Tab / Shift+Tab to move focus forward/backward through the single-line text boxes and
//!   the dropdown, in the order they appear on screen.
//! - Tab into the multi-line text box: it should insert an actual tab character instead of
//!   moving focus (its own keyboard handler owns Tab in that one case).
//! - Click "Open modal", then Tab/Shift+Tab: focus should stay confined to the modal's two
//!   text boxes and never leak back out to the fields behind it.

use bevy::prelude::*;
use woodpecker_ui::prelude::*;

#[derive(Component, PartialEq, Default, Debug, Clone, Reflect)]
#[reflect(Component, DiffableProp, PartialEq)]
struct ModalOpen(bool);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WoodpeckerUIPlugin::default())
        .add_systems(Startup, startup)
        .run();
}

fn label(text: &str) -> impl Bundle + Clone {
    (
        Element,
        WoodpeckerStyle {
            font_size: 14.0,
            color: Srgba::WHITE.into(),
            margin: Edge::all(0.0).bottom(4.0),
            ..Default::default()
        },
        WidgetRender::Text {
            content: text.into(),
        },
    )
}

fn field_style() -> WoodpeckerStyle {
    WoodpeckerStyle {
        width: Units::Pixels(320.0),
        margin: Edge::all(0.0).bottom(16.0),
        ..Default::default()
    }
}

fn startup(mut commands: Commands, mut ui_context: ResMut<WoodpeckerContext>) {
    commands.spawn((Camera2d, WoodpeckerView));

    let root_widget = ui_context.spawn_root(&mut commands);
    commands.entity(*root_widget).insert(
        WidgetChildren::default()
            .with_child::<ScrollContextProvider>((
                ScrollContextProvider::default(),
                WoodpeckerStyle {
                    width: Units::Percentage(100.0),
                    height: Units::Percentage(100.0),
                    ..Default::default()
                },
                WidgetChildren::default().with_child::<ScrollBox>((
                    ScrollBox::default(),
                    PassedChildren(
                        WidgetChildren::default().with_child::<Element>((
                            Element,
                            WoodpeckerStyle {
                                flex_direction: WidgetFlexDirection::Column,
                                padding: Edge::all(24.0),
                                width: Units::Percentage(100.0),
                                ..Default::default()
                            },
                            WidgetChildren::default()
                                .with_child::<Element>(label("First name (Tab stop 1)"))
                                .with_key("first_name_label")
                                .with_child::<TextBox>((
                                    TextBox {
                                        initial_value: "Ada".into(),
                                        ..Default::default()
                                    },
                                    field_style(),
                                ))
                                .with_key("first_name_field")
                                .with_child::<Element>(label("Last name (Tab stop 2)"))
                                .with_key("last_name_label")
                                .with_child::<TextBox>((
                                    TextBox {
                                        initial_value: "Lovelace".into(),
                                        ..Default::default()
                                    },
                                    field_style(),
                                ))
                                .with_key("last_name_field")
                                .with_child::<Element>(label(
                                    "Notes -- multi-line, Tab inserts a tab character here \
                                     instead of moving focus",
                                ))
                                .with_key("notes_label")
                                .with_child::<TextBox>((
                                    TextBox {
                                        multi_line: true,
                                        ..Default::default()
                                    },
                                    WoodpeckerStyle {
                                        height: Units::Pixels(80.0),
                                        ..field_style()
                                    },
                                ))
                                .with_key("notes_field")
                                .with_child::<Element>(label("Favorite color (Tab stop 3)"))
                                .with_key("color_label")
                                .with_child::<Dropdown>((
                                    Dropdown {
                                        current_value: "Blue".into(),
                                        list: vec!["Red".into(), "Green".into(), "Blue".into()],
                                    },
                                    field_style(),
                                ))
                                .with_key("color_field")
                                .with_child::<ModalDemo>(ModalDemo)
                                .with_key("modal_demo"),
                        )),
                    ),
                )),
            ))
            .with_child::<OverlayRootWidget>(OverlayRootWidget),
    );
}

#[derive(Widget, Component, Reflect, PartialEq, Default, Debug, Clone)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(render)]
#[require(WoodpeckerStyle, WidgetChildren)]
struct ModalDemo;

fn render(
    current_widget: Res<CurrentWidget>,
    mut commands: Commands,
    mut hooks: ResMut<HookHelper>,
    mut query: Query<&mut WidgetChildren, With<ModalDemo>>,
    state_query: Query<&ModalOpen>,
) {
    let Ok(mut children) = query.get_mut(**current_widget) else {
        return;
    };

    let state_entity = hooks.use_state(&mut commands, *current_widget, ModalOpen(false));
    let Ok(state) = state_query.get(state_entity) else {
        return;
    };
    let is_open = state.0;

    let current_widget = *current_widget;

    // Built separately (rather than inline in the `Modal` bundle below) so the "Close" click
    // observer can be scoped to the Close button itself -- `WidgetChildren::observe` attaches
    // to whichever child was *last added* at the point it's called. Attaching it after the
    // whole `Modal` child instead would catch every click that bubbles up through the modal's
    // content (e.g. focusing one of the trapped text boxes), closing it unintentionally.
    let mut modal_content = WidgetChildren::default();
    modal_content
        .add::<Element>(label(
            "Tab/Shift+Tab stays inside this modal -- it never reaches the fields behind it.",
        ))
        .add_key("instructions");
    modal_content
        .add::<TextBox>((
            TextBox {
                initial_value: "Trapped field 1".into(),
                ..Default::default()
            },
            field_style(),
        ))
        .add_key("field1");
    modal_content
        .add::<TextBox>((
            TextBox {
                initial_value: "Trapped field 2".into(),
                ..Default::default()
            },
            field_style(),
        ))
        .add_key("field2");
    modal_content
        .add::<WButton>((
            WButton,
            WidgetChildren::default().with_child::<Element>(label("Close")),
        ))
        .add_key("close_button");
    modal_content.observe(
        current_widget,
        move |_: On<Pointer<Click>>, mut state_query: Query<&mut ModalOpen>| {
            let Ok(mut state) = state_query.get_mut(state_entity) else {
                return;
            };
            state.0 = false;
        },
    );

    *children = WidgetChildren::default();
    children.add::<WButton>((
        WButton,
        WidgetChildren::default().with_child::<Element>(label("Open modal")),
    ));
    children.observe(
        current_widget,
        move |_: On<Pointer<Click>>, mut state_query: Query<&mut ModalOpen>| {
            let Ok(mut state) = state_query.get_mut(state_entity) else {
                return;
            };
            state.0 = true;
        },
    );
    children.add::<Modal>((
        Modal {
            title: "Focus-trapped modal".into(),
            visible: is_open,
            ..Default::default()
        },
        PassedChildren(WidgetChildren::default().with_child::<Element>((
            Element,
            WoodpeckerStyle {
                flex_direction: WidgetFlexDirection::Column,
                padding: Edge::all(16.0),
                width: Units::Percentage(100.0),
                ..Default::default()
            },
            modal_content,
        ))),
    ));

    children.apply(current_widget.as_parent());
}
