use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, Fields};

const DERIVE_BOUND_E: &str = "CmdArg can only be derived for tuple enum variants with no fields";

#[proc_macro_derive(CmdArg)]
pub fn cmdarg_macro_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();

    impl_cmdarg_macro(&ast)
}

fn impl_cmdarg_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let data = match &ast.data {
        Data::Enum(data) => data,
        _ => panic!("{DERIVE_BOUND_E}"),
    };

    let mut labels = Vec::with_capacity(data.variants.len());
    
    let branches = data
        .variants
        .iter()
        .map(|variant| {
            let ident = &variant.ident;
            let label = ident.to_string().to_lowercase();

            labels.push(label.clone());

            match &variant.fields {
                Fields::Unit => {
                    quote!{
                        #label => Ok(Self::#ident),
                    }
                }
                _ => panic!("{DERIVE_BOUND_E}"),
            }
        })
        .collect::<Vec<_>>();

    labels.iter_mut().for_each(|s| *s = format!("`{s}`"));

    let mismatch_e = match labels.len() {
        0 => String::from("invalid option"),
        1 => format!("invalid option, expects {}", labels.first().unwrap()),
        _ => {
            let last = labels.pop().unwrap();
            *labels.last_mut().unwrap() = format!("{} or {}", labels.last().unwrap(), last);
            format!("expects one of {}", labels.join(", "))
        }
    };


    let expanded = quote! {
        impl argp::FromArgValue for #name {
            fn from_arg_value(value: &std::ffi::OsStr) -> Result<Self, String> {
                match value.to_string_lossy().as_ref() {
                    #(#branches)*
                    _ => Err(String::from(#mismatch_e))
                }
            }
        }
    };

    expanded.into()
}
