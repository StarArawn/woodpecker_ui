use proc_macro::TokenStream;
use proc_macro2::Span;
use proc_macro_error::proc_macro_error;
use quote::quote;
use syn::Ident;

#[proc_macro_error]
#[proc_macro_derive(Widget, attributes(widget_systems, auto_update))]
pub fn widget_macro(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    let struct_identifier = &input.ident;

    const ATTR_ERROR_MESSAGE: &str = r#"
The `auto_update` and `widget_systems` attributes are the only supported arguments

= help: use `#[auto_update(render)] or #[widget_systems(update, render)]`
"#;

    // `systems.0` is the optional update system (defaults to always-false); `systems.1` is
    // the required render function name.
    let mut systems: (Option<proc_macro2::TokenStream>, Option<String>) = (None, None);
    let mut is_auto_update = false;

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
    }

    let render = if let Some(render) = systems.1 {
        Ident::new(&render, Span::call_site())
    } else {
        panic!("{}", ATTR_ERROR_MESSAGE);
    };

    // Re-render decisions come from `diffing::diff_widget_entity`, so all that's left to
    // generate here is the (optional) hot-reload check.
    #[cfg(feature = "hotreload")]
    if is_auto_update {
        systems.0 = Some(quote! {
            |mut old_pointer: bevy::prelude::Local<u64>| -> bool {
                let hot_fn = dioxus_devtools::subsecond::HotFn::current(#render);
                let new_ptr = hot_fn.ptr_address().0;
                if new_ptr != *old_pointer {
                    *old_pointer = new_ptr;
                    return true;
                }
                false
            }
        });
    }
    #[cfg(not(feature = "hotreload"))]
    let _ = is_auto_update;

    let update = if let Some(update) = systems.0 {
        update
    } else {
        quote! { || false }
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
                    *pat_type.pat = pat;
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
