use bevy::prelude::*;
use woodpecker_ui::prelude::CalendarDate;

#[derive(Debug, Reflect, Clone, PartialEq)]
pub struct UserRecord {
    pub name: String,
    pub email: String,
    pub role: String,
    pub active: bool,
    pub avatar_color: Color,
    /// Performance rating, 0..=5 in half-star increments.
    pub rating: f32,
    pub joined: CalendarDate,
    /// A short freeform note -- long enough that the editor modal clips it (see
    /// `users_page::build_editor_form`'s use of `Clip`).
    pub notes: String,
}

/// The full user list. Wrapped in `WatchedResource<Users>` (see `dashboard.rs`) so widgets
/// that display it re-render whenever an inline edit (role dropdown, active toggle) or a
/// modal save changes it.
#[derive(Resource, Reflect, Clone, PartialEq, Default, Deref, DerefMut)]
pub struct Users(pub Vec<UserRecord>);

/// Which user row (if any) the edit modal is currently open for.
#[derive(Resource, Reflect, Clone, Copy, PartialEq, Default)]
pub struct EditingUser(pub Option<usize>);

/// The current text in the topbar search box -- watched by `UsersTable` so it can fuzzy-
/// filter the list live as the user types.
#[derive(Resource, Reflect, Clone, PartialEq, Default)]
pub struct UserSearchQuery(pub String);

/// The current page (0-indexed) of the filtered/sorted user list -- watched by `UsersTable` so
/// it can slice the visible rows, and driven by `Pagination`'s own row below the table.
#[derive(Resource, Reflect, Clone, Copy, PartialEq, Default)]
pub struct UserPage(pub usize);

/// Whether the users table uses reduced row padding -- toggled by a `ToggleButton` above the
/// table (see `users_page::build_users_page`).
#[derive(Resource, Reflect, Clone, Copy, PartialEq, Default)]
pub struct CompactRows(pub bool);

/// Which workspace-wide permissions are granted vs. available -- watched by the Settings
/// page's `PermissionsPanel`, driven by its `TransferList`.
#[derive(Resource, Reflect, Clone, PartialEq)]
pub struct PermissionAssignment {
    pub available: Vec<String>,
    pub granted: Vec<String>,
}

impl Default for PermissionAssignment {
    fn default() -> Self {
        Self {
            available: vec![
                "Manage billing".into(),
                "Delete workspace".into(),
                "Manage integrations".into(),
            ],
            granted: vec!["Invite members".into(), "Export data".into()],
        }
    }
}

pub const ROLES: [&str; 3] = ["Admin", "Editor", "Viewer"];

const FIRST_NAMES: [&str; 20] = [
    "Ava", "Marcus", "Priya", "Diego", "Sofia", "Noah", "Liam", "Emma", "Olivia", "Ethan", "Mia",
    "Lucas", "Zoe", "Ryan", "Nora", "Kevin", "Grace", "Ivan", "Layla", "Omar",
];
const LAST_NAMES: [&str; 20] = [
    "Thompson",
    "Chen",
    "Patel",
    "Alvarez",
    "Kowalski",
    "Williams",
    "Garcia",
    "Nguyen",
    "Muller",
    "Rossi",
    "Johansson",
    "Kim",
    "Silva",
    "Novak",
    "Haddad",
    "Fischer",
    "Dubois",
    "Andersson",
    "Yamamoto",
    "Costa",
];
#[allow(
    clippy::approx_constant,
    reason = "0.318 is a color channel, not pi-related"
)]
const AVATAR_PALETTE: [Color; 8] = [
    Color::srgb(0.373, 0.51, 0.965),
    Color::srgb(0.29, 0.729, 0.494),
    Color::srgb(0.918, 0.702, 0.031),
    Color::srgb(0.937, 0.373, 0.373),
    Color::srgb(0.68, 0.475, 0.902),
    Color::srgb(0.376, 0.725, 0.847),
    Color::srgb(0.945, 0.522, 0.318),
    Color::srgb(0.478, 0.816, 0.518),
];

const NOTES_POOL: [&str; 5] = [
    "Joined via SSO integration during the Q1 migration. Primary on-call contact for platform \
     incidents affecting the billing pipeline.",
    "Originally onboarded as a contractor, converted to full-time after the Q3 review. Prefers \
     async updates over standing meetings.",
    "Manages the relationship with our largest enterprise customer. Escalate anything touching \
     that account through them first.",
    "Recently transferred from the infrastructure team. Still ramping up on the billing \
     domain, so loop in a second reviewer on payment-related changes.",
    "Founding team member. Has context on most of the legacy systems that predate the current \
     architecture docs.",
];

/// Generates 100 users -- `FIRST_NAMES`/`LAST_NAMES` are paired via a coprime stride (7 is
/// coprime with 20) so every one of the 100 combinations is distinct (no duplicate
/// name/email, which would otherwise collide as duplicate list keys).
pub fn seed_users() -> Users {
    let mut users = Vec::with_capacity(100);
    for i in 0..100 {
        let first = FIRST_NAMES[i % FIRST_NAMES.len()];
        let last = LAST_NAMES[(i * 7 + i / FIRST_NAMES.len()) % LAST_NAMES.len()];
        let role = match i % 10 {
            0 => ROLES[0],
            1..=4 => ROLES[1],
            _ => ROLES[2],
        };
        // Spreads join dates across 2021-01 through 2024-12 without ever landing past the
        // 28th (safe for every month, no per-month day-count table needed for demo data).
        let month_index = (i * 3) % 48;
        users.push(UserRecord {
            name: format!("{first} {last}"),
            email: format!("{}.{}@acme.dev", first.to_lowercase(), last.to_lowercase()),
            role: role.into(),
            active: i % 5 != 0,
            avatar_color: AVATAR_PALETTE[i % AVATAR_PALETTE.len()],
            rating: 2.0 + (i % 7) as f32 * 0.5,
            joined: CalendarDate::new(
                2021 + (month_index / 12) as i32,
                (month_index % 12) as u8 + 1,
                1 + (i % 28) as u8,
            ),
            notes: NOTES_POOL[i % NOTES_POOL.len()].into(),
        });
    }
    Users(users)
}

/// Monthly recurring revenue for the last 12 months, in thousands, this year vs. last year --
/// feeds the two-series "Revenue Trend" `LineChart` on the Overview page.
pub fn seed_revenue_trend() -> (Vec<f32>, Vec<f32>) {
    let this_year = vec![
        312.0, 328.0, 341.0, 335.0, 358.0, 372.0, 369.0, 391.0, 402.0, 418.0, 445.0, 482.9,
    ];
    let last_year = vec![
        248.0, 255.0, 267.0, 271.0, 279.0, 288.0, 291.0, 299.0, 305.0, 314.0, 322.0, 331.0,
    ];
    (this_year, last_year)
}

#[derive(Debug, Reflect, Clone, PartialEq)]
pub struct ActivityEntry {
    pub actor: String,
    pub action: String,
    pub target: String,
    pub time_ago: String,
}

pub fn seed_activity() -> Vec<ActivityEntry> {
    vec![
        ActivityEntry {
            actor: "Ava Thompson".into(),
            action: "deployed".into(),
            target: "billing-service v2.4.1".into(),
            time_ago: "2m ago".into(),
        },
        ActivityEntry {
            actor: "Marcus Chen".into(),
            action: "merged".into(),
            target: "PR #482: rework auth middleware".into(),
            time_ago: "18m ago".into(),
        },
        ActivityEntry {
            actor: "Priya Patel".into(),
            action: "commented on".into(),
            target: "Incident #113: elevated latency".into(),
            time_ago: "41m ago".into(),
        },
        ActivityEntry {
            actor: "System".into(),
            action: "scaled".into(),
            target: "worker pool to 12 nodes".into(),
            time_ago: "1h ago".into(),
        },
        ActivityEntry {
            actor: "Noah Williams".into(),
            action: "invited".into(),
            target: "sofia.kowalski@acme.dev".into(),
            time_ago: "3h ago".into(),
        },
        ActivityEntry {
            actor: "Diego Alvarez".into(),
            action: "revoked access for".into(),
            target: "legacy-reporting-key".into(),
            time_ago: "5h ago".into(),
        },
        ActivityEntry {
            actor: "Sofia Kowalski".into(),
            action: "updated".into(),
            target: "billing plan for Acme Corp".into(),
            time_ago: "1d ago".into(),
        },
    ]
}
