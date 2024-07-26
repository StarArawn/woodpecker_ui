use proc_macro::TokenStream;
use proc_macro2::Span;
use proc_macro_error::proc_macro_error;
use quote::quote;
use syn::Ident;

#[proc_macro_error]
#[proc_macro_derive(Widget, attributes(widget_systems, auto_update, diff))]
pub fn widget_macro(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    let struct_identifier = &input.ident;

    const ATTR_ERROR_MESSAGE: &str = r#"
The `systems` attribute is the only supported argument

= help: use `#[widget_systems(update, render)]`
"#;

    let mut systems: (Option<proc_macro2::TokenStream>, Option<String>) = (None, None);
    let mut is_auto_update = false;
    let mut diff_components = vec![];
    for attr in input.attrs.iter() {
        if attr.path().is_ident("widget_systems") {
            let list = attr.meta.require_list().expect(ATTR_ERROR_MESSAGE);
            let system_names = list.tokens.to_string();
            let split = system_names.split(',').collect::<Vec<_>>();
            let system_id = split
                .first()
                .expect(ATTR_ERROR_MESSAGE)
                .to_string()
                .replace(' ', "");
            let ident = Ident::new(&system_id, Span::call_site());
            systems.0 = Some(quote! {
                #ident
            });
            systems.1 = Some(
                split
                    .get(1)
                    .expect(ATTR_ERROR_MESSAGE)
                    .to_string()
                    .replace(' ', ""),
            );
        }
        if attr.path().is_ident("auto_update") {
            let list = attr.meta.require_list().expect(ATTR_ERROR_MESSAGE);
            let system_names = list.tokens.to_string();
            let split = system_names.split(',').collect::<Vec<_>>();

            systems.1 = Some(
                split
                    .first()
                    .expect(ATTR_ERROR_MESSAGE)
                    .to_string()
                    .replace(' ', ""),
            );
            is_auto_update = true;
        }

        if attr.path().is_ident("diff") && is_auto_update {
            let list = attr.meta.require_list().expect(ATTR_ERROR_MESSAGE);
            let system_names = list.tokens.to_string();
            let split = system_names.split(',').collect::<Vec<_>>();
            if split.is_empty() {
                panic!("{ATTR_ERROR_MESSAGE}");
            }
            for component in split {
                diff_components.push(component.to_string().replace(' ', ""));
            }
        }
    }

    if is_auto_update {
        let diff_components = diff_components
            .into_iter()
            .map(|c| Ident::new(&c, Span::call_site()))
            .collect::<Vec<_>>();
        let component_names_a = diff_components
            .iter()
            .enumerate()
            .map(|(i, ident)| Ident::new(&format!("a_{i}{ident}"), Span::call_site()))
            .collect::<Vec<_>>();
        let component_names_b = diff_components
            .iter()
            .enumerate()
            .map(|(i, ident)| Ident::new(&format!("b_{i}{ident}"), Span::call_site()))
            .collect::<Vec<_>>();

        let length = component_names_a.len();

        let component_diff = component_names_a
            .clone()
            .iter()
            .zip(component_names_b.clone())
            .enumerate()
            .map(|(i, (a, b))| {
                let andand = if length == 1 || i >= length - 1 {
                    None
                } else {
                    Some(quote! { && })
                };
                quote! {
                    #a != #b #andand
                }
            })
            .collect::<Vec<_>>();

        let component_diff = quote! {
            #(#component_diff)*
        };

        let struct_ident_string = struct_identifier.clone().to_string();

        systems.0 = Some(quote! {
            |mut commands: Commands, current_widget: Res<CurrentWidget>, mut hook_helper: ResMut<HookHelper>, query_a: Query<(Entity, #(&#diff_components, )*), Without<PreviousWidget>>, query_b: Query<(Entity, #(&#diff_components, )*), With<PreviousWidget>>,| {
                let Ok((entity, #(#component_names_a,)*)) = query_a.get(**current_widget) else {
                    error!("Woodpecker UI: WARNING! you are likely attempting to diff a component on the widget {} that does not exist!", #struct_ident_string);
                    return false;
                };
                let previous_widget_entity = hook_helper.get_previous_widget(&mut commands, *current_widget);
                // Replace old previous widget components with new ones
                commands.entity(previous_widget_entity).insert((
                    #(#component_names_a.clone(),)*
                ));

                let Ok((entity, #(#component_names_b,)*)) = query_b.get(previous_widget_entity) else {
                    // Probably means we mounted(created) so we should re-render!
                    return true;
                };

                let diff_result = #component_diff;
                diff_result
            }
        });
    }

    let systems = if let Some(update) = systems.0 {
        if let Some(render) = systems.1 {
            let render: Ident = Ident::new(&render, Span::call_site());
            quote! {
                fn update() -> impl bevy::prelude::System<In = (), Out = bool>
                where
                    Self: Sized,
                {
                    bevy::prelude::IntoSystem::into_system(#update)
                }

                fn render() -> impl bevy::prelude::System<In = (), Out = ()>
                where
                    Self: Sized,
                {
                    bevy::prelude::IntoSystem::into_system(#render)
                }
            }
        } else {
            panic!("{}", ATTR_ERROR_MESSAGE);
        }
    } else {
        quote! {}
    };

    quote! {
        #[automatically_derived]
        impl Widget for #struct_identifier {
            #systems
        }
    }
    .into()
}
