use bevy::prelude::*;
use woodpecker_ui::prelude::*;

/// A loot rarity tier. `theme::rarity_color()` maps this onto the real 5-color palette used
/// on `ItemSlot`'s border; `badge_variant()` below is a separate, best-effort mapping for the
/// small rarity pill `Badge` shown in tooltips/detail panes.
#[derive(Debug, Reflect, Clone, Copy, PartialEq, Eq)]
pub enum ItemRarity {
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
}

impl ItemRarity {
    pub fn label(self) -> &'static str {
        match self {
            ItemRarity::Common => "Common",
            ItemRarity::Uncommon => "Uncommon",
            ItemRarity::Rare => "Rare",
            ItemRarity::Epic => "Epic",
            ItemRarity::Legendary => "Legendary",
        }
    }

    pub fn badge_variant(self) -> BadgeVariant {
        match self {
            ItemRarity::Common => BadgeVariant::Neutral,
            ItemRarity::Uncommon => BadgeVariant::Success,
            ItemRarity::Rare => BadgeVariant::Info,
            ItemRarity::Epic => BadgeVariant::Warning,
            ItemRarity::Legendary => BadgeVariant::Danger,
        }
    }
}

#[derive(Debug, Reflect, Clone, PartialEq)]
pub struct InventoryItem {
    pub id: u64,
    pub name: String,
    pub rarity: ItemRarity,
    pub glyph: char,
    pub icon_color: Color,
    pub item_type: String,
    pub stat_lines: Vec<String>,
}

/// The player's loot -- reactive: a "Loot Item" demo button pushes new items, equip/unequip
/// moves ids between here and `Equipment`.
#[derive(Resource, Reflect, Clone, PartialEq, Default, Deref, DerefMut)]
pub struct Inventory(pub Vec<InventoryItem>);

/// Which inventory item (if any) is selected in the Inventory panel's detail pane.
///
/// Deliberately not `Deref` -- `WatchedResource<SelectedItem>`'s own `.0` field access would
/// compare against `Option<u64>` directly and fail to type-check. `.get()` sidesteps that via
/// auto-deref to this inherent method.
#[derive(Resource, Reflect, Clone, Copy, PartialEq, Default)]
pub struct SelectedItem(pub Option<u64>);

impl SelectedItem {
    pub fn get(&self) -> Option<u64> {
        self.0
    }
}

/// Currently-equipped item ids, by slot. `Option<u64>` indexes into `Inventory`.
#[derive(Resource, Reflect, Clone, Copy, PartialEq, Default)]
pub struct Equipment {
    pub weapon: Option<u64>,
    pub armor: Option<u64>,
    pub helm: Option<u64>,
    pub boots: Option<u64>,
    pub ring1: Option<u64>,
    pub ring2: Option<u64>,
}

/// Core vitals -- reactive: mutated by HUD demo buttons (Take Damage/Heal/Use Mana/Sprint)
/// and by `tick_stamina_regen`; drives every `StatBar` in the HUD.
#[derive(Resource, Reflect, Clone, Copy, PartialEq)]
pub struct PlayerVitals {
    pub health: f32,
    pub health_max: f32,
    pub mana: f32,
    pub mana_max: f32,
    pub stamina: f32,
    pub stamina_max: f32,
    pub xp: f32,
    pub xp_max: f32,
    pub level: u32,
    pub gold: u32,
}

impl Default for PlayerVitals {
    fn default() -> Self {
        Self {
            health: 0.0,
            health_max: 1.0,
            mana: 0.0,
            mana_max: 1.0,
            stamina: 0.0,
            stamina_max: 1.0,
            xp: 0.0,
            xp_max: 1.0,
            level: 1,
            gold: 0,
        }
    }
}

impl PlayerVitals {
    /// Applies XP, handling overflow into a level-up (carrying the remainder XP forward).
    /// Returns the new level if a level-up occurred.
    pub fn grant_xp(&mut self, amount: f32) -> Option<u32> {
        self.xp += amount;
        let mut leveled = None;
        while self.xp >= self.xp_max {
            self.xp -= self.xp_max;
            self.level += 1;
            self.xp_max *= 1.25;
            leveled = Some(self.level);
        }
        leveled
    }
}

#[derive(Debug, Reflect, Clone, PartialEq)]
pub struct AbilitySlotData {
    pub name: String,
    pub glyph: char,
    pub icon_color: Color,
    pub cooldown_total: f32,
    pub cooldown_remaining: f32,
}

/// The ability hotbar -- reactive: `tick_cooldowns` counts `cooldown_remaining` down every
/// frame, `AbilitySlot` starts a cooldown on click.
#[derive(Resource, Reflect, Clone, PartialEq, Default, Deref, DerefMut)]
pub struct HotbarState(pub Vec<AbilitySlotData>);

pub(crate) fn tick_cooldowns(time: Res<Time>, mut hotbar: ResMut<HotbarState>) {
    let dt = time.delta_secs();
    for slot in hotbar.0.iter_mut() {
        if slot.cooldown_remaining > 0.0 {
            slot.cooldown_remaining = (slot.cooldown_remaining - dt).max(0.0);
        }
    }
}

/// Slow, constant stamina regeneration -- a real game would gate this on "not sprinting";
/// kept simple here since sprinting is a one-shot demo-button drain, not a held state.
pub(crate) fn tick_stamina_regen(time: Res<Time>, mut vitals: ResMut<PlayerVitals>) {
    if vitals.stamina < vitals.stamina_max {
        vitals.stamina = (vitals.stamina + vitals.stamina_max * 0.08 * time.delta_secs())
            .min(vitals.stamina_max);
    }
}

#[derive(Debug, Reflect, Clone, PartialEq)]
pub struct BuffDebuff {
    pub id: u64,
    pub label: String,
    pub variant: BadgeVariant,
    pub tooltip: String,
    pub remaining: f32,
}

/// Active buffs/debuffs -- reactive: demo actions push entries, `tick_buffs` counts down
/// and removes expired ones.
#[derive(Resource, Reflect, Clone, PartialEq, Default)]
pub struct BuffState {
    pub buffs: Vec<BuffDebuff>,
    pub next_id: u64,
}

impl BuffState {
    pub fn push(
        &mut self,
        label: impl Into<String>,
        variant: BadgeVariant,
        tooltip: impl Into<String>,
        duration_secs: f32,
    ) {
        let id = self.next_id;
        self.next_id += 1;
        self.buffs.push(BuffDebuff {
            id,
            label: label.into(),
            variant,
            tooltip: tooltip.into(),
            remaining: duration_secs,
        });
    }
}

pub(crate) fn tick_buffs(time: Res<Time>, mut buffs: ResMut<BuffState>) {
    let dt = time.delta_secs();
    buffs.buffs.retain_mut(|b| {
        b.remaining -= dt;
        b.remaining > 0.0
    });
}

#[derive(Debug, Reflect, Clone, PartialEq)]
pub struct PartyMember {
    pub name: String,
    pub initials: String,
    pub avatar_color: Color,
    pub hp_fraction: f32,
    pub role: String,
}

/// Party frames -- reactive: a demo button damages one member's `hp_fraction`.
#[derive(Resource, Reflect, Clone, PartialEq, Default, Deref, DerefMut)]
pub struct PartyState(pub Vec<PartyMember>);

#[derive(Debug, Reflect, Clone, Copy, PartialEq, Eq)]
pub enum QuestKind {
    Main,
    Side,
}

#[derive(Debug, Reflect, Clone, PartialEq)]
pub struct Objective {
    pub label: String,
    pub current: u32,
    pub target: u32,
}

#[derive(Debug, Reflect, Clone, PartialEq)]
pub struct Quest {
    pub id: u64,
    pub title: String,
    pub kind: QuestKind,
    pub objectives: Vec<Objective>,
    pub completed: bool,
}

impl Quest {
    /// Fraction complete across all objectives combined (used for a per-quest ProgressBar).
    pub fn progress(&self) -> f32 {
        let (current, target): (u32, u32) = self
            .objectives
            .iter()
            .map(|o| (o.current, o.target))
            .fold((0, 0), |(c, t), (oc, ot)| (c + oc, t + ot));
        if target == 0 {
            1.0
        } else {
            (current as f32 / target as f32).clamp(0.0, 1.0)
        }
    }
}

/// The quest log -- reactive: demo "Advance"/"Turn In" buttons mutate objective progress.
#[derive(Resource, Reflect, Clone, PartialEq, Default, Deref, DerefMut)]
pub struct Quests(pub Vec<Quest>);

#[derive(Debug, Reflect, Clone, PartialEq)]
pub struct GuildMember {
    pub name: String,
    pub rank: String,
    pub level: u32,
    pub score: u32,
    pub class: String,
}

/// Static seed data -- `GuildRoster` widget (guild_panel.rs) re-sorts its own local copy in
/// response to `TableSort`, rather than mutating this resource.
#[derive(Resource, Reflect, Clone, PartialEq, Default, Deref, DerefMut)]
pub struct GuildRoster(pub Vec<GuildMember>);

/// Which floating menu windows are currently open -- reactive: watched by every window's
/// display-toggle wrapper in game_menu.rs, flipped by the HUD's quick-launch buttons.
#[derive(Resource, Reflect, Clone, Copy, PartialEq, Default)]
pub struct PanelVisibility {
    pub inventory: bool,
    pub character: bool,
    pub quests: bool,
    pub settings: bool,
    pub guild: bool,
}

/// Reactive: read back live by the HUD's vitals cluster (accent_color), a cross-panel link
/// from Settings to HUD.
#[derive(Resource, Reflect, Clone, Copy, PartialEq)]
pub struct GameSettings {
    pub master_volume: f32,
    pub music_volume: f32,
    pub graphics_preset_index: usize,
    pub difficulty_index: usize,
    pub show_damage_numbers: bool,
    pub invert_mouse_y: bool,
    pub accent_color: Color,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            master_volume: 0.8,
            music_volume: 0.6,
            graphics_preset_index: 2,
            difficulty_index: 1,
            show_damage_numbers: false,
            invert_mouse_y: false,
            accent_color: Color::srgb(0.686, 0.541, 0.212),
        }
    }
}

pub const GRAPHICS_PRESETS: [&str; 4] = ["Low", "Medium", "High", "Ultra"];
pub const DIFFICULTIES: [&str; 4] = ["Easy", "Normal", "Hard", "Nightmare"];

/// `Some(new_level)` when a level-up just happened -- reactive, drives `LevelUpCelebration`'s
/// `Modal::visible`.
#[derive(Resource, Reflect, Clone, Copy, PartialEq, Default)]
pub struct LevelUpFlag(pub Option<u32>);

impl LevelUpFlag {
    pub fn get(&self) -> Option<u32> {
        self.0
    }
}

pub fn seed_player_vitals() -> PlayerVitals {
    PlayerVitals {
        health: 420.0,
        health_max: 420.0,
        mana: 180.0,
        mana_max: 220.0,
        stamina: 100.0,
        stamina_max: 100.0,
        xp: 340.0,
        xp_max: 500.0,
        level: 12,
        gold: 1_284,
    }
}

pub fn seed_hotbar() -> HotbarState {
    HotbarState(vec![
        AbilitySlotData {
            name: "Slash".into(),
            glyph: 'S',
            icon_color: Color::srgb(0.75, 0.75, 0.8),
            cooldown_total: 1.5,
            cooldown_remaining: 0.0,
        },
        AbilitySlotData {
            name: "Fireball".into(),
            glyph: 'F',
            icon_color: Color::srgb(0.9, 0.4, 0.15),
            cooldown_total: 6.0,
            cooldown_remaining: 0.0,
        },
        AbilitySlotData {
            name: "Heal".into(),
            glyph: 'H',
            icon_color: Color::srgb(0.35, 0.75, 0.4),
            cooldown_total: 12.0,
            cooldown_remaining: 0.0,
        },
        AbilitySlotData {
            name: "Shield Wall".into(),
            glyph: 'W',
            icon_color: Color::srgb(0.4, 0.55, 0.85),
            cooldown_total: 20.0,
            cooldown_remaining: 0.0,
        },
        AbilitySlotData {
            name: "Meteor".into(),
            glyph: 'M',
            icon_color: Color::srgb(0.85, 0.2, 0.2),
            cooldown_total: 45.0,
            cooldown_remaining: 0.0,
        },
    ])
}

pub fn seed_party() -> PartyState {
    PartyState(vec![
        PartyMember {
            name: "Brannor".into(),
            initials: "BR".into(),
            avatar_color: Color::srgb(0.55, 0.35, 0.2),
            hp_fraction: 1.0,
            role: "Tank".into(),
        },
        PartyMember {
            name: "Elowen".into(),
            initials: "EL".into(),
            avatar_color: Color::srgb(0.35, 0.7, 0.55),
            hp_fraction: 0.85,
            role: "Healer".into(),
        },
        PartyMember {
            name: "Kestrel".into(),
            initials: "KE".into(),
            avatar_color: Color::srgb(0.7, 0.3, 0.3),
            hp_fraction: 0.6,
            role: "DPS".into(),
        },
    ])
}

const ITEM_ICON_PALETTE: [Color; 6] = [
    Color::srgb(0.75, 0.75, 0.8),
    Color::srgb(0.8, 0.6, 0.3),
    Color::srgb(0.4, 0.55, 0.85),
    Color::srgb(0.85, 0.35, 0.35),
    Color::srgb(0.5, 0.8, 0.5),
    Color::srgb(0.7, 0.5, 0.85),
];

/// 14 items spanning all 5 rarities -- ids assigned via a stride so they stay distinct, since
/// `inventory_panel.rs` keys its item loop with `.with_key(item.id)`.
pub fn seed_inventory() -> Inventory {
    let defs: [(&str, ItemRarity, char, &str, &[&str]); 14] = [
        (
            "Rusty Shortsword",
            ItemRarity::Common,
            'S',
            "Weapon",
            &["+4 Attack"],
        ),
        (
            "Traveler's Cloak",
            ItemRarity::Common,
            'C',
            "Armor",
            &["+3 Defense"],
        ),
        (
            "Leather Boots",
            ItemRarity::Common,
            'B',
            "Boots",
            &["+2 Defense", "+3 Speed"],
        ),
        (
            "Oakheart Bow",
            ItemRarity::Uncommon,
            'W',
            "Weapon",
            &["+9 Attack", "+2 Crit"],
        ),
        (
            "Ranger's Hood",
            ItemRarity::Uncommon,
            'H',
            "Helm",
            &["+5 Defense", "+4 Perception"],
        ),
        (
            "Band of Vigor",
            ItemRarity::Uncommon,
            'R',
            "Ring",
            &["+15 Max Health"],
        ),
        (
            "Frostbrand Blade",
            ItemRarity::Rare,
            'S',
            "Weapon",
            &["+18 Attack", "+6% Frost Dmg"],
        ),
        (
            "Stormcaller Staff",
            ItemRarity::Rare,
            'F',
            "Weapon",
            &["+22 Magic Power"],
        ),
        (
            "Guardian Plate",
            ItemRarity::Rare,
            'A',
            "Armor",
            &["+20 Defense", "+8 Max Health"],
        ),
        (
            "Ember Signet",
            ItemRarity::Rare,
            'R',
            "Ring",
            &["+10% Fire Dmg"],
        ),
        (
            "Void-Touched Greaves",
            ItemRarity::Epic,
            'B',
            "Boots",
            &["+14 Defense", "+9 Speed"],
        ),
        (
            "Wyrmscale Helm",
            ItemRarity::Epic,
            'H',
            "Helm",
            &["+16 Defense", "+12 Max Mana"],
        ),
        (
            "Dawnbringer",
            ItemRarity::Legendary,
            'S',
            "Weapon",
            &["+40 Attack", "+15% Crit", "On-hit: Heal 5%"],
        ),
        (
            "Crown of the Ancients",
            ItemRarity::Legendary,
            'H',
            "Helm",
            &["+30 Defense", "+50 Max Health", "+20 Max Mana"],
        ),
    ];
    let items = defs
        .iter()
        .enumerate()
        .map(
            |(i, (name, rarity, glyph, item_type, stats))| InventoryItem {
                id: (i as u64 + 1) * 7,
                name: (*name).into(),
                rarity: *rarity,
                glyph: *glyph,
                icon_color: ITEM_ICON_PALETTE[i % ITEM_ICON_PALETTE.len()],
                item_type: (*item_type).into(),
                stat_lines: stats.iter().map(|s| (*s).into()).collect(),
            },
        )
        .collect();
    Inventory(items)
}

pub fn seed_quests() -> Quests {
    Quests(vec![
        Quest {
            id: 1,
            title: "The Fallen Watchtower".into(),
            kind: QuestKind::Main,
            objectives: vec![
                Objective {
                    label: "Defeat the tower guardian".into(),
                    current: 0,
                    target: 1,
                },
                Objective {
                    label: "Recover the sealed relic".into(),
                    current: 0,
                    target: 1,
                },
            ],
            completed: false,
        },
        Quest {
            id: 2,
            title: "Whispers in the Deep".into(),
            kind: QuestKind::Main,
            objectives: vec![Objective {
                label: "Explore the sunken ruins".into(),
                current: 2,
                target: 5,
            }],
            completed: false,
        },
        Quest {
            id: 3,
            title: "Wolf Pelts for the Tanner".into(),
            kind: QuestKind::Side,
            objectives: vec![Objective {
                label: "Collect wolf pelts".into(),
                current: 3,
                target: 6,
            }],
            completed: false,
        },
        Quest {
            id: 4,
            title: "A Letter Undelivered".into(),
            kind: QuestKind::Side,
            objectives: vec![Objective {
                label: "Deliver the letter to Harbor Town".into(),
                current: 1,
                target: 1,
            }],
            completed: false,
        },
        Quest {
            id: 5,
            title: "Herbalist's Request".into(),
            kind: QuestKind::Side,
            objectives: vec![Objective {
                label: "Gather moonpetal herbs".into(),
                current: 0,
                target: 4,
            }],
            completed: false,
        },
    ])
}

pub fn seed_guild_roster() -> GuildRoster {
    let names = [
        "Brannor",
        "Elowen",
        "Kestrel",
        "Thane",
        "Yara",
        "Osric",
        "Mireille",
        "Duskwalker",
        "Aeliana",
        "Grimm",
        "Selwyn",
        "Ashara",
    ];
    let classes = ["Warrior", "Mage", "Rogue", "Cleric", "Ranger"];
    let ranks = [
        "Leader", "Officer", "Officer", "Member", "Member", "Member", "Member",
    ];
    let members = names
        .iter()
        .enumerate()
        .map(|(i, name)| GuildMember {
            name: (*name).into(),
            rank: ranks[i % ranks.len()].into(),
            level: 8 + ((i * 3) % 15) as u32,
            score: 1200 - (i as u32 * 47) % 900,
            class: classes[i % classes.len()].into(),
        })
        .collect();
    GuildRoster(members)
}

/// A few slots pre-filled -- ids must match entries in `seed_inventory()` (item id = index*7+7).
pub fn seed_equipment() -> Equipment {
    Equipment {
        weapon: Some(7),
        boots: Some(21),
        ..Default::default()
    }
}

/// "Power" trend over the last 10 levels -- feeds the Character panel's `LineChart`.
pub fn seed_power_trend() -> Vec<f32> {
    vec![
        120.0, 148.0, 175.0, 210.0, 238.0, 275.0, 312.0, 348.0, 390.0, 431.0,
    ]
}

/// Increments on every "Loot Item" demo button press -- there's no `rand` dependency, so
/// `roll_loot` uses this as a deterministic-but-varied index into `LOOT_TEMPLATES` instead.
#[derive(Resource, Reflect, Clone, Copy, PartialEq, Default)]
pub struct LootCounter(pub u64);

const LOOT_TEMPLATES: [(&str, ItemRarity, char, &str, &str); 8] = [
    (
        "Cracked Buckler",
        ItemRarity::Common,
        'A',
        "Armor",
        "+3 Defense",
    ),
    (
        "Weathered Dagger",
        ItemRarity::Common,
        'S',
        "Weapon",
        "+5 Attack",
    ),
    (
        "Hunter's Ring",
        ItemRarity::Uncommon,
        'R',
        "Ring",
        "+4 Perception",
    ),
    (
        "Silverleaf Boots",
        ItemRarity::Uncommon,
        'B',
        "Boots",
        "+6 Speed",
    ),
    (
        "Runed Warhammer",
        ItemRarity::Rare,
        'S',
        "Weapon",
        "+16 Attack",
    ),
    (
        "Sapphire Circlet",
        ItemRarity::Rare,
        'H',
        "Helm",
        "+10 Max Mana",
    ),
    (
        "Bloodroot Amulet",
        ItemRarity::Epic,
        'R',
        "Ring",
        "+12% Lifesteal",
    ),
    (
        "Sunfire Blade",
        ItemRarity::Legendary,
        'S',
        "Weapon",
        "+35 Attack, +10% Fire Dmg",
    ),
];

/// Produces a new, uniquely-id'd item from `LOOT_TEMPLATES`, cycling through them (with a
/// stride so consecutive rolls don't repeat the same template back-to-back for a small pool).
pub fn roll_loot(counter: &mut LootCounter) -> InventoryItem {
    let i = counter.0 as usize;
    counter.0 += 1;
    let (name, rarity, glyph, item_type, stat_line) =
        LOOT_TEMPLATES[(i * 3) % LOOT_TEMPLATES.len()];
    InventoryItem {
        id: 1000 + i as u64,
        name: name.into(),
        rarity,
        glyph,
        icon_color: ITEM_ICON_PALETTE[i % ITEM_ICON_PALETTE.len()],
        item_type: item_type.into(),
        stat_lines: vec![stat_line.into()],
    }
}
