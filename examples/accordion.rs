use bevy::prelude::*;
use woodpecker_ui::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WoodpeckerUIPlugin::default())
        .add_systems(Startup, startup)
        .run();
}

fn body_text(content: &str) -> impl Bundle + Clone {
    (
        Element,
        WoodpeckerStyle {
            font_size: 13.0,
            color: Srgba::new(0.72, 0.75, 0.8, 1.0).into(),
            text_wrap: TextWrap::WordOrGlyph,
            ..Default::default()
        },
        WidgetRender::Text {
            content: content.into(),
        },
    )
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
            .with_child::<Accordion>((
                Accordion {
                    mode: AccordionMode::Single,
                },
                PassedChildren(
                    WidgetChildren::default()
                        .with_child::<AccordionItem>((
                            AccordionItem {
                                key: "shipping".into(),
                                label: "Shipping".into(),
                            },
                            PassedChildren(WidgetChildren::default().with_child::<Element>(
                                body_text("Orders ship within 2 business days."),
                            )),
                        ))
                        .with_key("shipping")
                        .with_child::<AccordionItem>((
                            AccordionItem {
                                key: "returns".into(),
                                label: "Returns".into(),
                            },
                            PassedChildren(WidgetChildren::default().with_child::<Element>(
                                body_text("Returns are accepted within 30 days of delivery."),
                            )),
                        ))
                        .with_key("returns")
                        .with_child::<AccordionItem>((
                            AccordionItem {
                                key: "warranty".into(),
                                label: "Warranty".into(),
                            },
                            PassedChildren(WidgetChildren::default().with_child::<Element>(
                                body_text("All products include a 1 year limited warranty."),
                            )),
                        ))
                        .with_key("warranty"),
                ),
            ))
            .with_key("faq")
            .with_observe(root_widget, |trigger: On<Change<AccordionChanged>>| {
                info!(
                    "Accordion item {:?} expanded={}",
                    trigger.data.key, trigger.data.expanded
                );
            }),
    ));
}
