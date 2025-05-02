use super::TabContext;
use crate::prelude::*;
use bevy::prelude::*;
// use bevy_mod_picking::{
//     events::{Click, Pointer},
//     prelude::On,
// };

/// Tab button
#[derive(Widget, Component, Clone, PartialEq, Reflect)]
#[auto_update(render)]
#[props(TabButton)]
#[context(TabContext)]
pub struct TabButton {
    /// The index(it should match the index of the content)
    pub index: usize,
    /// The title of the tab.
    pub title: String,
    /// Inactive styles
    pub inactive_styles: ButtonStyles,
    /// Active styles
    pub active_styles: ButtonStyles,
}

impl Default for TabButton {
    fn default() -> Self {
        let base_styles = WoodpeckerStyle {
            align_items: Some(WidgetAlignItems::Center),
            height: 52.0.into(),
            font_size: 24.0,
            color: Color::WHITE,
            padding: Edge::all(0.0).left(16.0).right(16.0),
            ..Default::default()
        };
        Self {
            index: Default::default(),
            title: Default::default(),
            inactive_styles: ButtonStyles {
                normal: WoodpeckerStyle {
                    background_color: colors::BACKGROUND_LIGHT,
                    ..base_styles
                },
                hovered: WoodpeckerStyle {
                    background_color: colors::BACKGROUND_MID,
                    ..base_styles
                },
            },
            active_styles: ButtonStyles {
                normal: WoodpeckerStyle {
                    background_color: colors::BACKGROUND,
                    ..base_styles
                },
                hovered: WoodpeckerStyle {
                    background_color: colors::BACKGROUND,
                    ..base_styles
                },
            },
        }
    }
}

/// A tab button bundle
#[derive(Bundle, Default, Clone)]
pub struct TabButtonBundle {
    /// Tab button
    pub tab_button: TabButton,
    /// internal styles
    pub internal_styles: WoodpeckerStyle,
    /// internal children
    pub internal_children: WidgetChildren,
}

fn render(
    mut commands: Commands,
    current_widget: Res<CurrentWidget>,
    mut hooks: ResMut<HookHelper>,
    mut query: Query<(&TabButton, &mut WidgetChildren)>,
    context_query: Query<&TabContext>,
) {
    let Ok((tab_button, mut children)) = query.get_mut(**current_widget) else {
        return;
    };

    let context_entity = hooks.use_context(&mut commands, *current_widget, TabContext::default());

    let Ok(context) = context_query.get(context_entity) else {
        return;
    };

    let is_active = context.current_index == tab_button.index;

    // Actual button.
    let index = tab_button.index;
    children
        .add::<WButton>((WButtonBundle {
            button_styles: if is_active {
                tab_button.active_styles
            } else {
                tab_button.inactive_styles
            },
            children: WidgetChildren::default().with_child::<Element>((
                ElementBundle {
                    styles: WoodpeckerStyle {
                        font_size: if is_active {
                            tab_button.active_styles.normal.font_size
                        } else {
                            tab_button.inactive_styles.normal.font_size
                        },
                        color: if is_active {
                            tab_button.active_styles.normal.color
                        } else {
                            tab_button.inactive_styles.normal.color
                        },
                        ..Default::default()
                    },
                    ..Default::default()
                },
                WidgetRender::Text {
                    content: tab_button.title.clone(),
                    word_wrap: false,
                },
            )),
            ..Default::default()
        },))
        .observe(
            move |_trigger: Trigger<Pointer<Click>>, mut context_query: Query<&mut TabContext>| {
                let Ok(mut context) = context_query.get_mut(context_entity) else {
                    return;
                };
                context.current_index = index;
            },
        );

    children.apply(current_widget.as_parent());
}
