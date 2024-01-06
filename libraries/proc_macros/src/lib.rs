use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DataEnum, DeriveInput};

#[proc_macro_derive(NumStrEnum)]
pub fn num_str_enum(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let variants = match input.data {
        Data::Enum(DataEnum { variants, .. }) => variants,
        _ => panic!("#[derive(NumStrEnum)] is only defined for enums"),
    };

    let name = input.ident;

    let match_arms = variants.iter().enumerate().map(|(index, variant)| {
        let variant_name = &variant.ident;
        let index_u8 = index as u8;
        quote! { #index_u8 => Ok(#name::#variant_name), }
    });

    let value_arms = variants.iter().enumerate().map(|(index, variant)| {
        let index_u8 = index as u8;
        let variant_name = &variant.ident;
        quote! { #name::#variant_name => #index_u8, }
    });

    let error_name = quote::format_ident!("{}Error", name);

    let expanded = quote! {
        #[derive(Debug)]
        pub enum #error_name {
            InvalidString(String),
            InvalidValue(u8),
        }

        impl std::error::Error for #error_name {}

        impl std::fmt::Display for #error_name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                match self {
                    #error_name::InvalidString(value) => write!(f, "Invalid string: {}", value),
                    #error_name::InvalidValue(value) => write!(f, "Invalid value: {}", value),
                }
            }
        }

        impl std::str::FromStr for #name {
            type Err = #error_name;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s.parse::<u8>() {
                    Ok(num) => match num {
                        #(#match_arms)*
                        _ => Err(#error_name::InvalidValue(num)),
                    },
                    Err(_) => Err(#error_name::InvalidString(s.to_string())),
                }
            }
        }

        impl #name {
            pub fn value(&self) -> u8 {
                match self {
                    #(#value_arms)*
                }
            }
        }
    };

    TokenStream::from(expanded)
}
