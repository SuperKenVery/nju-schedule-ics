use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemStruct};

#[proc_macro_attribute]
pub fn register_school(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemStruct);
    let name = &input.ident;

    let expanded = quote! {
        #input

        inventory::submit! {
            crate::adapters::traits::SchoolRegistration {
                factory: |db| Box::pin(async move {
                    let adapter = #name::new(db).await;
                    std::sync::Arc::new(adapter) as std::sync::Arc<dyn crate::adapters::traits::School>
                })
            }
        }
    };

    TokenStream::from(expanded)
}
