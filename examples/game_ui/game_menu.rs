use bevy::prelude::*;
use woodpecker_ui::prelude::*;

use crate::character_panel::CharacterSheet;
use crate::guild_panel::GuildRoster;
use crate::inventory_panel::InventoryGrid;
use crate::mock_data::PanelVisibility;
use crate::quests_panel::QuestLog;
use crate::settings_panel::build_settings_panel;
use crate::theme;

#[derive(Reflect, Clone, Copy, PartialEq, Eq, Default, Debug)]
enum PanelKind {
    #[default]
    Inventory,
    Character,
    Quests,
    Settings,
    Guild,
}

impl PanelKind {
    fn is_visible(self, v: &PanelVisibility) -> bool {
        match self {
            PanelKind::Inventory => v.inventory,
            PanelKind::Character => v.character,
            PanelKind::Quests => v.quests,
            PanelKind::Settings => v.settings,
            PanelKind::Guild => v.guild,
        }
    }

    fn close(self, v: &mut PanelVisibility) {
        match self {
            PanelKind::Inventory => v.inventory = false,
            PanelKind::Character => v.character = false,
            PanelKind::Quests => v.quests = false,
            PanelKind::Settings => v.settings = false,
            PanelKind::Guild => v.guild = false,
        }
    }
}

/// Wraps a single window's content (passed in via `PassedChildren`) in a fresh
/// [`WoodpeckerWindow`], rebuilt every render so `WoodpeckerWindow::visible` always reflects
/// the current `PanelVisibility` -- rather than being frozen at whatever value happened to be
/// baked into a statically-constructed bundle. An earlier version of this widget instead built
/// the `WoodpeckerWindow` bundle once (in `floating_window`, called once at startup) and tried
/// to toggle visibility afterwards via a direct `Query<&mut WoodpeckerWindow>` write from here;
/// that write was silently clobbered every single frame, because `WidgetChildren`'s own
/// reconciliation re-inserts the *original* cloned bundle (with `visible: true` baked in from
/// startup) on every pass regardless of whether anything about it changed. Rebuilding the whole
/// bundle here, freshly, every render -- the same pattern every other reactive widget in this
/// crate already uses -- means the declared `visible` value is always the one reconciliation
/// actually applies. This can't be done by toggling `styles.display` on this widget's own
/// entity like an ordinary ancestor would: the window's actual rendered content is portaled to
/// `OverlayRoot`, so it's no longer a physical descendant this entity's style can cascade to.
#[derive(Widget, Component, Reflect, Clone, PartialEq, Default)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(panel_window_slot_render)]
#[require(WidgetChildren, WoodpeckerStyle, WatchedResource<PanelVisibility>)]
pub(crate) struct PanelWindowSlot {
    panel: PanelKind,
    title: String,
    initial_position: Vec2,
    width: f32,
}

/// A bundle for [`PanelWindowSlot`] -- mirrors `TabContentBundle`'s shape. `children` here is
/// just the window's raw inner content (e.g. `InventoryGrid`) -- `panel_window_slot_render`
/// wraps it in a `WoodpeckerWindow` itself every render.
#[derive(Bundle, Clone, Default)]
struct PanelWindowSlotBundle {
    slot: PanelWindowSlot,
    children: PassedChildren,
    internal_styles: WoodpeckerStyle,
    internal_children: WidgetChildren,
}

fn panel_window_slot_render(
    current_widget: Res<CurrentWidget>,
    mut query: Query<(
        &PanelWindowSlot,
        &WatchedResource<PanelVisibility>,
        &PassedChildren,
        &mut WidgetChildren,
    )>,
) {
    let Ok((slot, visibility, passed_children, mut children)) = query.get_mut(**current_widget)
    else {
        return;
    };

    let visible = slot.panel.is_visible(visibility);
    let current = *current_widget;

    let mut window_children = WidgetChildren::default();
    window_children.add::<WoodpeckerWindow>((
        WoodpeckerWindow {
            title: slot.title.clone(),
            initial_position: slot.initial_position,
            visible,
            window_styles: WoodpeckerStyle {
                width: slot.width.into(),
                ..theme::window_card_style()
            },
            // Must reserve room for this padding plus the window's own border (see
            // `window_width_for`), since taffy uses border-box sizing.
            children_styles: WoodpeckerStyle {
                padding: Edge::all(theme::SPACE_MD),
                ..Default::default()
            },
            ..Default::default()
        },
        PassedChildren(passed_children.0.clone()),
        window_title_children(current, &slot.title, slot.panel),
    ));

    *children = window_children;

    children.apply(current_widget.as_parent());
}

/// Builds a custom title row for a floating panel's `WoodpeckerWindow`: label text + a
/// flexible spacer + a small "✕" close button -- the only mechanism available, since
/// `WoodpeckerWindow` has no built-in close button or visibility prop.
fn window_title_children(
    current_widget: CurrentWidget,
    title: &str,
    panel: PanelKind,
) -> TitleChildren {
    let mut children = WidgetChildren::default();
    children.add::<Element>((
        Element,
        WoodpeckerStyle {
            font_size: 14.0,
            color: theme::TEXT_PRIMARY,
            text_wrap: TextWrap::None,
            flex_grow: 1.0,
            ..Default::default()
        },
        WidgetRender::Text {
            content: title.into(),
        },
    ));
    children.add_key("label");
    children.add::<Element>((
        Element,
        WoodpeckerStyle {
            width: 22.0.into(),
            height: 22.0.into(),
            justify_content: Some(WidgetAlignContent::Center),
            align_items: Some(WidgetAlignItems::Center),
            border_radius: Corner::all(theme::RADIUS_SM),
            color: theme::TEXT_MUTED,
            font_size: 13.0,
            ..Default::default()
        },
        WidgetChildren::default().with_child::<Element>((
            Element,
            WoodpeckerStyle {
                color: theme::TEXT_MUTED,
                font_size: 13.0,
                text_wrap: TextWrap::None,
                ..Default::default()
            },
            // Plain ASCII "X" -- the "✕" (U+2715) glyph isn't in the embedded Poppins font
            // and renders as a missing-glyph tofu box.
            WidgetRender::Text {
                content: "X".into(),
            },
        )),
        Pickable::default(),
    ));
    children.add_key("close");
    children.observe(
        current_widget,
        move |mut trigger: On<Pointer<Click>>, mut visibility: ResMut<PanelVisibility>| {
            trigger.propagate(false);
            panel.close(&mut visibility);
        },
    );
    children.hover_cursor(current_widget, SystemCursorIcon::Pointer);

    TitleChildren(children)
}

/// The window's own border width, from `theme::window_card_style()` -- factored out since
/// `window_width_for` below needs to know it too.
const WINDOW_BORDER_WIDTH: f32 = 2.0;

/// Computes the `WoodpeckerWindow` width so `content_width` fits inside its padded, bordered
/// content box. Taffy uses border-box sizing, so this has to account for both the window's
/// padding and its own border, or the content overflows past the border.
fn window_width_for(content_width: f32) -> f32 {
    content_width + theme::SPACE_MD * 2.0 + WINDOW_BORDER_WIDTH * 2.0
}

/// Caps `content` to a fixed height with its overflow scrollable -- used for panel content
/// (like `build_settings_panel`'s output) that isn't itself a widget with its own style to
/// attach a height cap to.
///
/// Width must be a concrete pixel value, not `Percentage(100.0)`: the window's body wrapper
/// is `Auto`-sized to its content, so a percentage here has no concrete ancestor to resolve
/// against and collapses to near-zero, shrinking the whole window down to a sliver.
fn scrollable_content(content: WidgetChildren, width: f32, height: f32) -> WidgetChildren {
    WidgetChildren::default().with_child::<Element>((
        Element,
        WoodpeckerStyle {
            width: width.into(),
            height: height.into(),
            ..Default::default()
        },
        theme::scrollable_panel(content),
    ))
}

fn floating_window(
    panel: PanelKind,
    title: &str,
    initial_position: Vec2,
    // An explicit width, rather than leaving the window `Auto`-sized: the title bar copies
    // the window's computed width each render, and an `Auto`-sized window can take a frame
    // or two to settle, leaving the title bar stuck at a too-small snapshot.
    width: f32,
    // Raw widget-tree content -- `WoodpeckerWindow::render()` already wraps this in an
    // `Element` sized from `window.children_styles`.
    content: WidgetChildren,
) -> impl Bundle + Clone {
    PanelWindowSlotBundle {
        slot: PanelWindowSlot {
            panel,
            title: title.into(),
            initial_position,
            width,
        },
        children: PassedChildren(content),
        ..Default::default()
    }
}

/// The shared `WindowingContextProvider` plus the 5 floating menu panels.
pub fn build_game_menu(current_widget: CurrentWidget) -> WidgetChildren {
    WidgetChildren::default().with_child::<WindowingContextProvider>(
        WidgetChildren::default()
            .with_child::<PanelWindowSlot>(floating_window(
                PanelKind::Inventory,
                "Inventory",
                // A small cascade near the center; `WoodpeckerWindow` clamps to the real
                // viewport on every render, so this only needs to be a reasonable default.
                Vec2::new(380.0, 140.0),
                window_width_for(420.0),
                WidgetChildren::default().with_child::<InventoryGrid>((InventoryGrid,)),
            ))
            .with_key("inventory")
            .with_child::<PanelWindowSlot>(floating_window(
                PanelKind::Character,
                "Character",
                Vec2::new(420.0, 170.0),
                window_width_for(420.0),
                WidgetChildren::default().with_child::<CharacterSheet>((CharacterSheet,)),
            ))
            .with_key("character")
            .with_child::<PanelWindowSlot>(floating_window(
                PanelKind::Quests,
                "Quest Log",
                Vec2::new(460.0, 200.0),
                window_width_for(420.0),
                WidgetChildren::default().with_child::<QuestLog>((QuestLog,)),
            ))
            .with_key("quests")
            .with_child::<PanelWindowSlot>(floating_window(
                PanelKind::Settings,
                "Settings",
                Vec2::new(400.0, 230.0),
                window_width_for(420.0),
                scrollable_content(build_settings_panel(current_widget), 420.0, 380.0),
            ))
            .with_key("settings")
            .with_child::<PanelWindowSlot>(floating_window(
                PanelKind::Guild,
                "Guild Roster",
                Vec2::new(440.0, 260.0),
                window_width_for(480.0),
                WidgetChildren::default().with_child::<GuildRoster>((GuildRoster,)),
            ))
            .with_key("guild"),
    )
}
