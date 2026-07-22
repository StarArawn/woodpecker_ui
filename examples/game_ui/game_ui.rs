mod ability_slot;
mod character_panel;
mod game_menu;
mod guild_panel;
mod hud;
mod inventory_panel;
mod level_up_modal;
mod mock_data;
mod quests_panel;
mod settings_panel;
mod stat_bar;
mod theme;

use bevy::prelude::*;
use woodpecker_ui::prelude::*;

use ability_slot::AbilitySlot;
use character_panel::CharacterSheet;
use game_menu::PanelWindowSlot;
use guild_panel::GuildRoster as GuildRosterWidget;
use hud::{BuffRow, PartyFrames, VitalsCluster};
use inventory_panel::{InventoryGrid, ItemSlot};
use level_up_modal::LevelUpCelebration;
use mock_data::*;
use quests_panel::QuestLog;
use settings_panel::AccentColorSwatch;
use stat_bar::StatBar;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WoodpeckerUIPlugin::default())
        .register_widget::<StatBar>()
        .register_widget::<AbilitySlot>()
        .register_widget::<VitalsCluster>()
        .register_widget::<BuffRow>()
        .register_widget::<PartyFrames>()
        .register_widget::<PanelWindowSlot>()
        .register_widget::<ItemSlot>()
        .register_widget::<InventoryGrid>()
        .register_widget::<CharacterSheet>()
        .register_widget::<QuestLog>()
        .register_widget::<GuildRosterWidget>()
        .register_widget::<LevelUpCelebration>()
        .register_widget::<AccentColorSwatch>()
        .register_watched_resource::<PlayerVitals>()
        .register_watched_resource::<HotbarState>()
        .register_watched_resource::<BuffState>()
        .register_watched_resource::<PartyState>()
        .register_watched_resource::<Inventory>()
        .register_watched_resource::<SelectedItem>()
        .register_watched_resource::<Equipment>()
        .register_watched_resource::<Quests>()
        .register_watched_resource::<GuildRoster>()
        .register_watched_resource::<PanelVisibility>()
        .register_watched_resource::<GameSettings>()
        .register_watched_resource::<LevelUpFlag>()
        .add_systems(Update, (tick_cooldowns, tick_buffs, tick_stamina_regen))
        .insert_resource(seed_player_vitals())
        .insert_resource(seed_hotbar())
        .insert_resource(BuffState::default())
        .insert_resource(seed_party())
        .insert_resource(seed_inventory())
        .insert_resource(SelectedItem::default())
        .insert_resource(seed_equipment())
        .insert_resource(seed_quests())
        .insert_resource(seed_guild_roster())
        .insert_resource(PanelVisibility::default())
        .insert_resource(GameSettings::default())
        .insert_resource(LevelUpFlag::default())
        .insert_resource(LootCounter::default())
        .add_systems(Startup, startup)
        .run();
}

fn startup(mut commands: Commands, mut ui_context: ResMut<WoodpeckerContext>) {
    commands.spawn((Camera2d, WoodpeckerView));

    let root_widget = ui_context.spawn_root(&mut commands);

    let hotbar_slots = seed_hotbar().0.len();

    commands.entity(*root_widget).insert(
        WidgetChildren::default()
            .with_child::<Element>((
                Element,
                WoodpeckerStyle::default(),
                hud::build_hud(root_widget, hotbar_slots),
            ))
            .with_key("hud")
            .with_child::<Element>((
                Element,
                WoodpeckerStyle::default(),
                game_menu::build_game_menu(root_widget),
            ))
            .with_key("game_menu")
            .with_child::<LevelUpCelebration>((LevelUpCelebration,))
            .with_key("level_up")
            .with_child::<ToastViewport>(ToastViewport)
            .with_key("toasts")
            .with_child::<OverlayRootWidget>(OverlayRootWidget),
    );
}
