use proc_macro::TokenStream;
use proc_macro2::Span;
use proc_macro_error::proc_macro_error;
use quote::quote;
use syn::{spanned::Spanned, Ident};

#[proc_macro_error]
#[proc_macro_derive(Widget, attributes(widget_systems, auto_update, props, state, context))]
pub fn widget_macro(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    let struct_identifier = &input.ident;

    const ATTR_ERROR_MESSAGE: &str = r#"
The `auto_update` and `widget_systems` attributes are the only supported arguments

= help: use `#[auto_update(render)] or #[widget_systems(update, render)]`
"#;

    let mut systems: (Option<proc_macro2::TokenStream>, Option<String>) = (None, None);
    let mut is_auto_update = false;
    let mut is_auto_diff_state = false;
    let mut is_auto_diff_context = false;
    let mut is_diff_props = false;
    let mut diff_props = vec![];
    let mut diff_state = vec![];
    let mut diff_context = vec![];

    let mut props_span = None;
    let mut state_span = None;
    let mut context_span = None;

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

        if attr.path().is_ident("props") && is_auto_update {
            let list = attr.meta.require_list().expect(ATTR_ERROR_MESSAGE);
            let system_names = list.tokens.to_string();
            let split = system_names.split(',').collect::<Vec<_>>();
            if split.is_empty() {
                return syn::Error::new(list.span(), ATTR_ERROR_MESSAGE)
                    .to_compile_error()
                    .into();
            }
            for component in split {
                diff_props.push(component.to_string().replace(' ', ""));
            }
            is_diff_props = true;
            props_span = Some(attr.path().get_ident().span());
        }
        if attr.path().is_ident("state") && is_auto_update {
            let list = attr.meta.require_list().expect(ATTR_ERROR_MESSAGE);
            let system_names = list.tokens.to_string();
            let split = system_names.split(',').collect::<Vec<_>>();
            if split.is_empty() {
                return syn::Error::new(list.span(), ATTR_ERROR_MESSAGE)
                    .to_compile_error()
                    .into();
            }
            for component in split {
                diff_state.push(component.to_string().replace(' ', ""));
            }
            is_auto_diff_state = true;
            state_span = Some(attr.path().get_ident().span());
        }

        if attr.path().is_ident("context") && is_auto_update {
            let list = attr.meta.require_list().expect(ATTR_ERROR_MESSAGE);
            let system_names = list.tokens.to_string();
            let split = system_names.split(',').collect::<Vec<_>>();
            if split.is_empty() {
                return syn::Error::new(list.span(), ATTR_ERROR_MESSAGE)
                    .to_compile_error()
                    .into();
            }
            for component in split {
                diff_context.push(component.to_string().replace(' ', ""));
            }
            is_auto_diff_context = true;
            context_span = Some(attr.path().get_ident().span());
        }
    }

    let render = if let Some(render) = systems.1 {
        Ident::new(&render, Span::call_site())
    } else {
        panic!("{}", ATTR_ERROR_MESSAGE);
    };

    if is_auto_update {
        if !is_diff_props {
            return syn::Error::new(input.span(), "`auto_update` attribute used but no props were specified please use #[props(Component)] and specify at least one component to diff.")
                .to_compile_error()
                .into();
        }

        let (prop_diff, prop_names_a, prop_names_b, prop_type_names) =
            get_diff(props_span.unwrap(), diff_props, true);

        if prop_type_names
            .iter()
            .any(|tn| tn.to_string().contains("Transition"))
        {
            let prop_name = prop_type_names
                .iter()
                .find(|tn| tn.to_string().contains("Transition"))
                .unwrap();

            return syn::Error::new(prop_name.span(), "Transitions are automatically diffed internally. As they are handled specially to avoid re-renders unless the animation starts/finishes. Please remove the Transition from the `props` attribute.")
            .to_compile_error()
            .into();
        }

        let (state_query_statements, state_query_lookups) = if is_auto_diff_state {
            let (compiler_error, state_names_a, state_names_b, state_type_names) =
                get_diff(state_span.unwrap(), diff_state, false);

            let state_names_a_query = state_names_a
                .iter()
                .map(|n| Ident::new(&format!("{}_query", n), Span::call_site()))
                .collect::<Vec<_>>();
            let state_names_b_query = state_names_b
                .iter()
                .map(|n| Ident::new(&format!("{}_query", n), Span::call_site()))
                .collect::<Vec<_>>();

            let state_type_names_string = state_type_names
                .iter()
                .map(|tn| tn.to_string())
                .collect::<Vec<_>>();

            (
                Some(quote! {
                    #compiler_error
                    #(#state_names_a_query: Query<&#state_type_names, Without<PreviousWidget>>,)*
                    #(#state_names_b_query: Query<&#state_type_names, With<PreviousWidget>>,)*
                }),
                Some(quote! {
                    #(
                        if let Some(state_entity) = hook_helper.get_state::<#state_type_names>(*current_widget) {
                            let Ok(#state_names_a) = #state_names_a_query.get(state_entity) else {
                                error!("Woodpecker UI: WARNING! you are likely attempting to diff a state component on the widget {} that does not exist!", #state_type_names_string);
                                return false;
                            };

                            // Replace old previous widget state component with new one
                            commands.entity(previous_widget_entity).insert(
                                #state_names_a.clone()
                            );

                            let Ok(#state_names_b) = #state_names_b_query.get(previous_widget_entity) else {
                                // Probably means we have fresh state created so we should re-render!
                                return true;
                            };

                            if #state_names_a != #state_names_b {
                                // State changed lets return true!
                                return true;
                            }
                        }
                    )*
                }),
            )
        } else {
            (None, None)
        };

        let (context_query_statements, context_query_lookups) = if is_auto_diff_context {
            let (compiler_error, context_names_a, context_names_b, context_type_names) =
                get_diff(context_span.unwrap(), diff_context, false);

            let context_names_a_query = context_names_a
                .iter()
                .map(|n| Ident::new(&format!("{}_query", n), Span::call_site()))
                .collect::<Vec<_>>();
            let context_names_b_query = context_names_b
                .iter()
                .map(|n| Ident::new(&format!("{}_query", n), Span::call_site()))
                .collect::<Vec<_>>();

            let context_type_names_string = context_type_names
                .iter()
                .map(|tn| tn.to_string())
                .collect::<Vec<_>>();

            (
                Some(quote! {
                    #compiler_error
                    #(#context_names_a_query: Query<&#context_type_names, Without<PreviousWidget>>,)*
                    #(#context_names_b_query: Query<&#context_type_names, With<PreviousWidget>>,)*
                }),
                Some(quote! {
                    #(
                        if let Some(context_entity) = hook_helper.get_context::<#context_type_names>(*current_widget) {
                            let Ok(#context_names_a) = #context_names_a_query.get(context_entity) else {
                                error!("Woodpecker UI: WARNING! you are likely attempting to diff a context component on the widget {} that does not exist!", #context_type_names_string);
                                return false;
                            };

                            // Replace old previous widget state component with new one
                            commands.entity(previous_widget_entity).insert(
                                #context_names_a.clone()
                            );

                            let Ok(#context_names_b) = #context_names_b_query.get(previous_widget_entity) else {
                                // Probably means we have fresh state created so we should re-render!
                                return true;
                            };

                            if #context_names_a != #context_names_b {
                                // State changed lets return true!
                                return true;
                            }
                        }
                    )*
                }),
            )
        } else {
            (None, None)
        };

        let struct_ident_string = struct_identifier.clone().to_string();

        let render = render.clone();
        systems.0 = Some(quote! {
            |
                mut commands: Commands,
                current_widget: Res<CurrentWidget>,
                mut hook_helper: ResMut<HookHelper>,
                child_query: Query<&WidgetChildren>,
                query_changed: Query<Entity, With<Mounted>>,
                query_a: Query<(Entity, #(&#prop_type_names, )*), Without<PreviousWidget>>,
                query_b: Query<(Entity, #(&#prop_type_names, )*), With<PreviousWidget>>,
                #state_query_statements
                #context_query_statements
                transition_query: Query<&Transition>,
                #[cfg(feature = "hotreload")]
                mut old_pointer: Local<u64>
            | {
                #[cfg(feature = "hotreload")] {
                    let hot_fn = dioxus_devtools::subsecond::HotFn::current(#render);
                    let new_ptr = hot_fn.ptr_address();
                    if new_ptr != *old_pointer {
                        *old_pointer = new_ptr;
                        return true;
                    }
                }

                // Ignore no children
                if let Ok(children) = child_query.get(**current_widget) {
                    if children.children_changed() {
                        return true;
                    }
                }

                /// Widget mount
                if query_changed.get(**current_widget).is_ok() {
                    commands.entity(**current_widget).remove::<Mounted>();
                    return true;
                }

                let Ok((entity, #(#prop_names_a,)*)) = query_a.get(**current_widget) else {
                    error!("Woodpecker UI: WARNING! you are likely attempting to diff a component on the widget {} that does not exist!", #struct_ident_string);
                    return false;
                };
                let previous_widget_entity = hook_helper.get_previous_widget(&mut commands, *current_widget);
                // Replace old previous widget components with new ones
                commands.entity(previous_widget_entity).insert((
                    #(#prop_names_a.clone(),)*
                ));

                #state_query_lookups

                #context_query_lookups

                let Ok((entity, #(#prop_names_b,)*)) = query_b.get(previous_widget_entity) else {
                    // Probably means we mounted(created) so we should re-render!
                    return true;
                };

                if let Ok(transition_a) = transition_query.get(**current_widget) {
                    commands.entity(previous_widget_entity).insert(transition_a.clone());
                    if let Ok(transition_b) = transition_query.get(previous_widget_entity) {
                        if transition_a.is_playing() != transition_b.is_playing() {
                            return true;
                        }
                    }
                }

                let diff_result = #prop_diff;
                diff_result
            }
        });
    }

    let update = if let Some(update) = systems.0 {
        update
    } else {
        quote! {}
    };

    let systems = quote! {
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
    };

    quote! {
        #[automatically_derived]
        impl Widget for #struct_identifier {
            #systems
        }
    }
    .into()
}

fn get_diff(
    error_span: Span,
    diff_items: Vec<String>,
    include_diff: bool,
) -> (proc_macro2::TokenStream, Vec<Ident>, Vec<Ident>, Vec<Ident>) {
    let mut diff_props = diff_items
        .iter()
        .map(|c| Ident::new(c, Span::call_site()))
        .collect::<Vec<_>>();

    diff_props.sort();
    diff_props.dedup();

    if diff_items.len() > 1 {
        let num_dups = diff_items.len() - diff_props.len();

        if num_dups > 0 {
            return (
                syn::Error::new(error_span, "You have duplicate components!").to_compile_error(),
                vec![],
                vec![],
                vec![],
            );
        }
    }

    let prop_names_a = diff_props
        .iter()
        .enumerate()
        .map(|(i, ident)| Ident::new(&format!("a_{i}{ident}"), Span::call_site()))
        .collect::<Vec<_>>();
    let prop_names_b = diff_props
        .iter()
        .enumerate()
        .map(|(i, ident)| Ident::new(&format!("b_{i}{ident}"), Span::call_site()))
        .collect::<Vec<_>>();

    let length = prop_names_a.len();

    let prop_diff = prop_names_a
        .clone()
        .iter()
        .zip(prop_names_b.clone())
        .enumerate()
        .map(|(i, (a, b))| {
            let or_op = if length == 1 || i >= length - 1 {
                None
            } else {
                Some(quote! { || })
            };
            quote! {
                #a != #b #or_op
            }
        })
        .collect::<Vec<_>>();

    (
        if include_diff {
            quote! {
                #(#prop_diff)*
            }
        } else {
            proc_macro2::TokenStream::new()
        },
        prop_names_a,
        prop_names_b,
        diff_props,
    )
}

#[proc_macro_error]
#[proc_macro_attribute]
#[cfg(feature = "hotreload")]
pub fn hot(_attr: TokenStream, func: TokenStream) -> TokenStream {
    use quote::ToTokens;
    use syn::{parse, ItemFn};

    let input_function: ItemFn = parse(func).unwrap();
    let func_name = input_function.sig.ident;
    let wrapped_input = input_function.sig.inputs;
    let block = input_function.block;

    let func_name_wrapped = Ident::new(&format!("{}_wrapped", func_name), func_name.span());

    let input_names = wrapped_input
        .iter()
        .filter_map(|fa| match fa {
            syn::FnArg::Receiver(_receiver) => None,
            syn::FnArg::Typed(pat_type) => {
                if let syn::Pat::Ident(pat_ident) = *pat_type.pat.clone() {
                    Some(pat_ident.ident.to_token_stream())
                } else {
                    None
                }
            }
        })
        .collect::<Vec<_>>();

    let input = wrapped_input
        .clone()
        .into_iter()
        .map(|mut fn_arg| {
            match &mut fn_arg {
                syn::FnArg::Receiver(_receiver) => {}
                syn::FnArg::Typed(pat_type) => {
                    let mut pat = *pat_type.pat.clone();
                    if let syn::Pat::Ident(pat_ident) = &mut pat {
                        pat_ident.mutability = None;
                    }
                    pat_type.pat = Box::new(pat);
                }
            }

            fn_arg
        })
        .collect::<Vec<_>>();

    quote! {
        fn #func_name(#(#input,)*) {
            dioxus_devtools::subsecond::HotFn::current(#func_name_wrapped).call((#(#input_names,)*))
        }

        fn #func_name_wrapped(#wrapped_input)
            #block

    }
    .into()
}
