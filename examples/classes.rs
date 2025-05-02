use bevy::prelude::*;
use bevy_vello::render::VelloView;
use woodpecker_ui::prelude::*;

/// A list of classes our widgets will use
/// Ideally these are scoped to a SINGLE widget.
/// As in styles should live with their widgets.
/// This example is primiarly meant to serve as a
/// resuable pattern. This also cuts down on boilerplate
/// within the widget spawning code itself. Again
/// this is not an endorsement for global styles rather
/// this serves as an example of how you might reference
/// multiple styles "classes" in a single widget.
/// This is useful for focus, hover, click style changes
/// and other things.
mod classes {
    #![allow(non_upper_case_globals)]
    use bevy::prelude::*;
    use woodpecker_ui::prelude::*;

    /// Styles for our main app widget.
    pub const app_styles: WoodpeckerStyle = WoodpeckerStyle {
        flex_direction: WidgetFlexDirection::Column,
        ..WoodpeckerStyle::DEFAULT
    };
    /// Red text styles
    pub const red_text: WoodpeckerStyle = WoodpeckerStyle {
        color: Color::Srgba(Srgba::RED),
        ..WoodpeckerStyle::DEFAULT
    };
    /// Blue text styles
    pub const blue_text: WoodpeckerStyle = WoodpeckerStyle {
        color: Color::Srgba(Srgba::BLUE),
        ..WoodpeckerStyle::DEFAULT
    };
    /// Green text styles
    pub const green_text: WoodpeckerStyle = WoodpeckerStyle {
        color: Color::Srgba(Srgba::GREEN),
        ..WoodpeckerStyle::DEFAULT
    };
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WoodpeckerUIPlugin::default())
        .add_systems(Startup, startup)
        .run();
}

fn startup(mut commands: Commands, mut ui_context: ResMut<WoodpeckerContext>) {
    commands.spawn((Camera2d, VelloView));

    let root = commands
        .spawn(WoodpeckerAppBundle {
            styles: classes::app_styles,
            children: WidgetChildren::default()
                .with_child::<Element>((
                    ElementBundle {
                        styles: classes::red_text,
                        ..Default::default()
                    },
                    WidgetRender::Text {
                        content: "Hello, I am red text!".into(),
                        word_wrap: false,
                    },
                ))
                .with_child::<Element>((
                    ElementBundle {
                        styles: classes::blue_text,
                        ..Default::default()
                    },
                    WidgetRender::Text {
                        content: "Hello, I am blue text!".into(),
                        word_wrap: false,
                    },
                ))
                .with_child::<Element>((
                    ElementBundle {
                        styles: classes::green_text,
                        ..Default::default()
                    },
                    WidgetRender::Text {
                        content: "Hello, I am green text!".into(),
                        word_wrap: false,
                    },
                )),
            ..Default::default()
        })
        .id();
    ui_context.set_root_widget(root);
}
