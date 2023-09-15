use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, Fields};

const DERIVE_BOUND_E: &str = "Command can only be derived for tuple enum variants with a single field";

#[proc_macro_derive(Command)]
pub fn command_macro_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();

    impl_command_macro(&ast)
}

fn impl_command_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let data = match &ast.data {
        Data::Enum(data) => data,
        _ => panic!("{DERIVE_BOUND_E}"),
    };
    
    let branches = data
        .variants
        .iter()
        .map(|variant| {
            let ident = &variant.ident;
            match &variant.fields {
                Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                    quote! {
                        Self::#ident(inner) => inner.run(),
                    }
                }
                _ => panic!("{DERIVE_BOUND_E}"),
            }
        })
        .collect::<Vec<_>>();

    let expanded = quote! {
        impl command_macro::CommandTrait for #name {
            fn run(&self) -> Result<(), Box<dyn Error>> {
                match self {
                    #(#branches)*
                }
            }
        }
    };
    expanded.into()
}
