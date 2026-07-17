use bevy::{ecs::reflect::AppTypeRegistry, prelude::*, reflect::ReflectRef};

use crate::hook_helper::HookHelper;

/// A generic reflection dump of every `#[reflect(Component)]`-registered component on `entity`,
/// **except** `WoodpeckerStyle` (the panel gives that its own editable form -- see
/// `style_editor` -- so it would otherwise show up twice). Struct-shaped components are broken
/// out one row per field (`field_at`/`name_at` via `ReflectRef::Struct`) rather than one big
/// pretty-printed blob, so a component with a dozen fields reads as a dozen short lines instead
/// of a wall of text; anything else (tuple structs, enums, plain values) falls back to a single
/// Debug-formatted line, since there's no meaningful further breakdown for those.
pub(crate) fn dump_entity(world: &World, entity: Entity) -> Vec<(String, String)> {
    let Ok(entity_ref) = world.get_entity(entity) else {
        return Vec::new();
    };
    let type_registry = world.resource::<AppTypeRegistry>().clone();
    let registry = type_registry.read();

    let mut out = Vec::new();
    for &component_id in entity_ref.archetype().components() {
        let Some(info) = world.components().get_info(component_id) else {
            continue;
        };
        let Some(type_id) = info.type_id() else {
            continue;
        };
        let Some(registration) = registry.get(type_id) else {
            continue;
        };
        let type_path = registration.type_info().type_path().to_string();
        if type_path.ends_with("::WoodpeckerStyle") {
            continue;
        }
        let Some(reflect_component) = registration.data::<ReflectComponent>() else {
            continue;
        };
        let Some(value) = reflect_component.reflect(entity_ref) else {
            continue;
        };
        let formatted = match value.reflect_ref() {
            ReflectRef::Struct(s) => {
                let mut lines = Vec::new();
                for i in 0..s.field_len() {
                    let name = s.name_at(i).unwrap_or("?");
                    match s.field_at(i) {
                        Some(field) => lines.push(format!("  {name}: {field:?}")),
                        None => lines.push(format!("  {name}: <unreadable>")),
                    }
                }
                lines.join("\n")
            }
            _ => format!("{value:?}"),
        };
        out.push((type_path, formatted));
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

fn format_section(title: &str, fields: &[(String, String)]) -> String {
    if fields.is_empty() {
        return String::new();
    }
    let mut out = format!("== {title} ==\n");
    for (name, value) in fields {
        out.push_str(name);
        out.push('\n');
        out.push_str(value);
        out.push_str("\n\n");
    }
    out
}

/// Like [`dump_entity`], but also dumps every `use_state`/`use_context` entity `entity` owns
/// or inherits (see `HookHelper`) -- these live on separate child entities, not on `entity`
/// itself, so they need their own dump section. Reuses `HookHelper::own_state_entities`/
/// `visible_context_entities`, the crate's own established pattern for this exact problem
/// (already used identically by `diffing::diff_widget_entity`).
pub(crate) fn dump_entity_and_state(world: &World, entity: Entity) -> String {
    let mut sections = vec![format_section(
        &format!("Entity {entity}"),
        &dump_entity(world, entity),
    )];

    if let Some(hook_helper) = world.get_resource::<HookHelper>() {
        let mut state_entities = hook_helper.own_state_entities(entity);
        for context_entity in hook_helper.visible_context_entities(entity) {
            if !state_entities.contains(&context_entity) {
                state_entities.push(context_entity);
            }
        }
        for state_entity in state_entities {
            let fields = dump_entity(world, state_entity);
            if !fields.is_empty() {
                sections.push(format_section(&format!("State {state_entity}"), &fields));
            }
        }
    }

    sections.retain(|s| !s.is_empty());
    sections.join("\n")
}
