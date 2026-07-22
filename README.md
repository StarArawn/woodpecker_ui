[![Crates.io](https://img.shields.io/crates/v/woodpecker_ui)](https://crates.io/crates/woodpecker_ui)
[![docs](https://docs.rs/woodpecker_ui/badge.svg)](https://docs.rs/woodpecker_ui/)
[![License](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](https://github.com/StarArawn/woodpecker_ui#license)
[![Crates.io](https://img.shields.io/crates/d/woodpecker_ui)](https://crates.io/crates/woodpecker_ui)

<h1>
    <p align="center">
    Woodpecker UI
    <p>
</h1>

Woodpecker UI is an ECS-first, reactive UI crate for the Bevy game engine. Widgets are just entities and components, layout is Taffy, rendering is [vello](https://github.com/linebender/bevy_vello), text is [Parley](https://github.com/linebender/parley), and the tree only re-renders the widgets that actually changed.

# Features
  - **ECS first** — widgets are entities/components, no external retained UI tree to keep in sync with the game world
  - **Plain Rust control flow** — `for`, `if`, `match` all just work; no template/macro DSL to learn
  - **[vello](https://github.com/linebender/bevy_vello) rendering** with box shadows, clipping, SVGs, and 9-patch images
  - **[Taffy](https://github.com/DioxusLabs/taffy)** flexbox + grid layouting
  - **[Parley](https://github.com/linebender/parley)** text layout, rich text, and multi-line text editing
  - **React-style hooks** — `use_state`, `use_context`, `use_effect`, `use_memo`, `use_previous`, `use_timer`, `use_interval`, `use_debounce`
  - **Theming** system with dark/light and custom themes
  - **Animation** — spring-based transitions and keyframe timelines
  - Drag & drop, gamepad navigation, and keyboard tab-focus navigation built in
  - Devtools and a `metrics` feature for inspecting the widget tree at runtime
  - Optional [Bevy BSN](https://github.com/bevyengine/bevy/issues/14437) scene support (`bevy_bsn` feature)
  - Experimental hot reloading via [Dioxus hot patching](https://github.com/DioxusLabs/dioxus)
  - 60+ ready-made widgets (see below) to get you started

# Widget catalog
  - **Inputs**: Button, IconButton, Checkbox, Radio Group, Toggle, Toggle Button, Slider, Number Input, Text Box, Combo Box, Dropdown, Date Picker, Color Picker, Rating
  - **Navigation**: App Bar, Bottom Navigation, Navigation Rail, Breadcrumbs, Tabs, Drawer, Pagination, Stepper, Menu
  - **Data display**: Table, Tree View, Virtual List, List, Line Chart, Timeline, Typography, Markdown (with syntax highlighting), Avatar, Badge, Chip, Image List
  - **Feedback & overlays**: Alert, Toast, Modal, Popover, Tooltip, Skeleton, Spinner, Progress Bar, Speed Dial
  - **Layout**: Element, Paper, Card, Divider, Clip, Scroll Box, Splitter, Masonry, Accordion, Transfer List, Windows

Run `cargo run --example gallery` for a Storybook-style, in-app catalog of every widget above plus a built-in guide covering architecture, hooks, styling/theming, layout, and composition.

### Running on desktop:
`cargo run --example todo`

Other showcase examples worth a look:
  - `cargo run --example gallery` — interactive widget catalog and guide
  - `cargo run --example dashboard` — a full admin dashboard app
  - `cargo run --example game_ui` — an RPG-style HUD/inventory/character screen UI

### Running on WASM:
1. `cargo install wasm-server-runner`
2. `RUSTFLAGS="--cfg=web_sys_unstable_apis" cargo run --example todo --target wasm32-unknown-unknown --release`
3. `wasm-server-runner target/wasm32-unknown-unknown/release/todo.wasm`

### Experimental hot reloading support
1. `cargo install dioxus-cli --version 0.7.0-alpha.0`
2. `dx serve --example counter --hotpatch --features="hotreload"`

Hot reloading is very lightweight and wont hinder your performance in release mode at all! Currently only the todo example is wired up for hot reloading but any widget render system can be hot reloaded with the #[hot] macro!

### Bevy BSN support
Enable the `bevy_bsn` feature to declare widget trees using Bevy's [BSN](https://github.com/bevyengine/bevy/issues/14437) scene syntax instead of (or alongside) the builder API. See [examples/bsn.rs](examples/bsn.rs) for a complete example.

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

    let root_widget = ui_context.spawn_root(&mut commands);
    commands
        .entity(*root_widget)
        .insert(WidgetChildren::default().with_child::<Element>((
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
            },
        )));
}
```

<details>
    <summary>Counter Example</summary>

```rust
use bevy::prelude::*;
use woodpecker_ui::prelude::*;

#[derive(Component, PartialEq, Default, Debug, Clone, Reflect)]
#[reflect(Component, DiffableProp, PartialEq)]
pub struct CounterState {
    count: u32,
}

#[derive(Widget, Component, Reflect, PartialEq, Default, Debug, Clone)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(render)]
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
                    },
                )),
            ))
            .with_observe(
                current_widget,
                move |_: On<Pointer<Click>>, mut query: Query<&mut CounterState>| {
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

    let root_widget = ui_context.spawn_root(&mut commands);
    commands
        .entity(*root_widget)
        .insert(WidgetChildren::default().with_child::<CounterWidget>((
            CounterWidget { initial_count: 0 },
            WoodpeckerStyle {
                width: Units::Percentage(100.0),
                ..Default::default()
            },
        )));
}
```
</details>

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
