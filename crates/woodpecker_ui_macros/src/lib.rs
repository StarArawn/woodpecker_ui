use proc_macro::TokenStream;
use proc_macro2::Span;
use proc_macro_error::proc_macro_error;
use quote::quote;
use syn::Ident;

#[proc_macro_error]
#[proc_macro_derive(Widget, attributes(widget_systems))]
pub fn widget_macro(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    let struct_identifier = &input.ident;

    const ATTR_ERROR_MESSAGE: &str = r#"
The `systems` attribute is the only supported argument

= help: use `#[systems(update, render)]`
"#;

    let mut systems: (Option<String>, Option<String>) = (None, None);
    for attr in input.attrs.iter() {
        if attr.path().is_ident("widget_systems") {
            let list = attr.meta.require_list().expect(ATTR_ERROR_MESSAGE);
            let system_names = list.tokens.to_string();
            let split = system_names.split(',').collect::<Vec<_>>();
            systems.0 = Some(
                split
                    .first()
                    .expect(ATTR_ERROR_MESSAGE)
                    .to_string()
                    .replace(' ', ""),
            );
            systems.1 = Some(
                split
                    .get(1)
                    .expect(ATTR_ERROR_MESSAGE)
                    .to_string()
                    .replace(' ', ""),
            );
        }
    }

    let systems = if let Some(update) = systems.0 {
        let update: Ident = Ident::new(&update, Span::call_site());
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
