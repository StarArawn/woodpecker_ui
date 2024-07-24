[![Crates.io](https://img.shields.io/crates/v/woodpecker_ui)](https://crates.io/crates/woodpecker_ui)
[![docs](https://docs.rs/woodpecker_ui/badge.svg)](https://docs.rs/woodpecker_ui/)
[![License](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](https://github.com/StarArawn/woodpecker_ui/blob/main/LICENSE)
[![Crates.io](https://img.shields.io/crates/d/woodpecker_ui)](https://crates.io/crates/woodpecker_ui)

<h1>
    <p align="center">
    Woodpecker UI
    <p>
</h1>

Woodpecker UI is a Bevy ECS driven user interface crate. Its designed to be easy to use and work seamlessly with the bevy game engine.

# Features
  - ECS **first** UI
  - Easy to use widget systems
  - Flexable UI rendering using [vello](https://github.com/linebender/bevy_vello)
  - [Taffy](https://github.com/DioxusLabs/taffy) layouting
  - [Cosmic Text](https://github.com/pop-os/cosmic-text) for text layouting
  - A few helper widgets to get you started

### Found a bug? Please open an issue!

## Basic Example(text.rs):
```rust
use bevy::prelude::*;
use bevy_mod_picking::DefaultPickingPlugins;
use woodpecker_ui::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WoodpeckerUIPlugin)
        .add_plugins(DefaultPickingPlugins)
        .add_systems(Startup, startup)
        .run();
}

fn startup(mut commands: Commands, mut ui_context: ResMut<WoodpeckerContext>) {
    commands.spawn(Camera2dBundle::default());

    let root = commands
        .spawn(WoodpeckerAppBundle {
            children: WidgetChildren::default().with_child::<Element>((
                ElementBundle {
                    styles: WoodpeckerStyle {
                        font_size: 50.0,
                        color: Srgba::RED.into(),
                        margin: Edge::all(10.0),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                WidgetRender::Text {
                    content: "Hello World! I am Woodpecker UI!".into(),
                    word_wrap: false,
                },
            )),
            ..Default::default()
        })
        .id();
    ui_context.set_root_widget(root);
}
```

# Q and A

## Q1. Why not use Bevy UI?
  1. Bevy UI rendering leaves a lot to be desired. Woodpecker UI uses vello a newer rendering system for UI's. It supports everything that I was looking for in a UI renderer.
  2. Bevy UI is designed to be an immediate mode UI similar to egui. Woodpecker UI is reactive and only changes the widget tree when a widget changes and in the future will only render to the screen when changed.

## Q2. Why not use one of the other UI libraries out there?
  1. A lot of times they don't integrate into the ECS very nicely. They tend to want ownership of the data which means it must live outside of bevy's ECS world. I have problems with this.
  2. Non-Rust syntax. Woodpecker UI uses rust syntax for everything, a for loop is a for loop, an if statement is an if statement. There is no custom wrappers for these things in Woodpecker UI. Which makes writing code a lot easier! See: [examples/todo/list.rs Line 53](https://github.com/StarArawn/woodpecker_ui/tree/main/examples/todo/list.rs#L53)
  3. They use Bevy UI. See the Bevy UI section above.

## Q3. What about Kayak UI?
You might notice the syntax used here is quite similar to Kayak UI, but Kayak UI suffered from overly complicated internals. It made contributing to Kayak UI much too difficult and caused quite a few fundamental bugs. In Woodpecker UI I took what made Kayak UI great and made the backend much much simpler. As an example the primiary system that runs the UI was over 1k lines in Kayak and in Woodpecker its less than 200! This should help foster collaborative development and encourage people to help fix bugs!

## Q4. Why not wait for the next-gen Bevy UI? Why make your own?
  1. There is no timeline for when this might come out.
  2. There are a lot of conflicting opinions about how the next-gen Bevy UI should work. In my opinion there isn't a clear direction(yet although its starting to form). How does it render things? What about input eventing? I hope/believe this will change for the better!
  3. So far I'm personally not a huge fan of using scenes and also the new BSN macro. From what I've seen it has some problems around not using rust syntax, data management, and although you can opt out of using BSN you cannot opt out of using scenes and entity patches for UI. Although thats not completely clear yet.
  4. I apparently really like writing UI crates.

## Q5. Should I use Woodpecker UI?
I would look at the features and like any other crate that you pick you should weigh your options and pick the one best suited to your needs. I don't claim that Woodpecker UI will fit any need and its really up to the individual to decide.
