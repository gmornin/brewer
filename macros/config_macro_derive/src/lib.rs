use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_derive(Config)]
pub fn config_macro_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();

    impl_config_macro(&ast)
}

fn impl_config_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;

    if !name.to_string().ends_with("Config") {
        panic!("struct name must end with `Config`");
    }

    let label = &name.to_string()[..name.to_string().len() - 6].to_lowercase();

    let expanded = quote! {
        impl config_macro::ConfigTrait for #name {
            const NAME: &'static str = #label;
        }
    };

    expanded.into()
}
