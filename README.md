[![Crates.io](https://img.shields.io/crates/v/woodpecker_ui)](https://crates.io/crates/woodpecker_ui)
[![docs](https://docs.rs/woodpecker_ui/badge.svg)](https://docs.rs/woodpecker_ui/)
[![License](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](https://github.com/StarArawn/woodpecker_ui#license)
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
  - [Parley](https://github.com/linebender/parley) for text layouting and editing
  - A few helper widgets to get you started


### Running on desktop:
`cargo run --example todo`

### Running on WASM:
1. `cargo install wasm-server-runner`
2. `RUSTFLAGS="--cfg=web_sys_unstable_apis" cargo run --example todo --target wasm32-unknown-unknown --release`
3. `wasm-server-runner target/wasm32-unknown-unknown/release/todo.wasm`

### Experimental hot reloading support
1. `cargo install dioxus-cli --version 0.7.0-alpha.0`
2. `dx serve --example counter --hotpatch --features="hotreload"`

Hot reloading is very lightweight and wont hinder your performance in release mode at all! Currently only the todo example is wired up for hot reloading but any widget render system can be hot reloaded with the #[hot] macro!

### Found a bug? Please open an issue!

### Basic Example [examples/text.rs](examples/text.rs):
```rust
use bevy::prelude::*;
use woodpecker_ui::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WoodpeckerUIPlugin::default())
        .add_systems(Startup, startup)
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
            WidgetChildren::default().with_child::<Element>((
                Element,
                WoodpeckerStyle {
                    font_size: 50.0,
                    color: Srgba::RED.into(),
                    margin: Edge::all(10.0),
                    font: Some(font.id()),
                    ..Default::default()
                },
                WidgetRender::Text {
                    content: "Hello World! I am Woodpecker UI!".into(),
                    word_wrap: false,
                },
            )),
        ))
        .id();
    ui_context.set_root_widget(root);
}
```

<details>
    <summary>Counter Example</summary>

```rust
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
```
</details>


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

## License

Woodpecker UI is free, open source and permissively licensed!
Except where noted (below and/or in individual files), all code in this repository is dual-licensed under either:

* MIT License ([LICENSE-MIT](LICENSE-MIT) or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))
* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))

at your option.
This means you can select the license you prefer!
This dual-licensing approach is the de-facto standard in the Rust ecosystem and there are [very good reasons](https://github.com/bevyengine/bevy/issues/2373) to include both.

Some of the engine's code carries additional copyright notices and license terms due to their external origins.
These are generally BSD-like, but exact details vary by crate:
If the README of a crate contains a 'License' header (or similar), the additional copyright notices and license terms applicable to that crate will be listed.
The above licensing requirement still applies to contributions to those crates, and sections of those crates will carry those license terms.
