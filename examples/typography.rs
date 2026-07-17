use bevy::prelude::*;
use woodpecker_ui::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WoodpeckerUIPlugin::default())
        .add_systems(Startup, startup)
        .run();
}

fn row(text: &str, variant: TypographyVariant) -> impl Bundle + Clone {
    (Typography {
        text: text.into(),
        variant,
        color: None,
    },)
}

fn startup(mut commands: Commands, mut ui_context: ResMut<WoodpeckerContext>) {
    commands.spawn((Camera2d, WoodpeckerView));

    let root_widget = ui_context.spawn_root(&mut commands);
    commands.entity(*root_widget).insert((
        WoodpeckerStyle {
            padding: Edge::all(20.0),
            flex_direction: WidgetFlexDirection::Column,
            align_items: Some(WidgetAlignItems::Start),
            gap: (0.0.into(), 8.0.into()),
            ..Default::default()
        },
        WidgetChildren::default()
            .with_child::<Typography>(row("Heading 1", TypographyVariant::H1))
            .with_key("h1")
            .with_child::<Typography>(row("Heading 2", TypographyVariant::H2))
            .with_key("h2")
            .with_child::<Typography>(row("Heading 3", TypographyVariant::H3))
            .with_key("h3")
            .with_child::<Typography>(row(
                "Body text, the default variant for ordinary copy.",
                TypographyVariant::Body,
            ))
            .with_key("body")
            .with_child::<Typography>(row(
                "Body small, for secondary/de-emphasized lines.",
                TypographyVariant::BodySmall,
            ))
            .with_key("body-small")
            .with_child::<Typography>(row(
                "Caption, the smallest legible size.",
                TypographyVariant::Caption,
            ))
            .with_key("caption"),
    ));
}
