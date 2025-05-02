use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use heck::ToSnakeCase;
use proc_macro2::{Ident as Ident2, Span as Span2, TokenStream as TokenStream2};
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::{LitStr, Token};

pub fn macro_impl(input: TokenStream2) -> syn::Result<TokenStream2> {
    let file_path: LitStr = syn::parse2(input)?;
    let span = file_path.span();
    let file_path = file_path.value();

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("set by cargo");
    let file_path = PathBuf::new().join(manifest_dir).join(file_path);

    let content = fs::read_to_string(&file_path).map_err(|err| syn::Error::new(span, err))?;
    let table: toml::Table = toml::from_str(&content).map_err(|err| syn::Error::new(span, err))?;

    let key =
        file_path.file_name().ok_or_else(|| syn::Error::new(span, "could not find file name"))?;
    let key = key.to_string_lossy().trim_end_matches(".toml").to_snake_case();
    let tokens = codegen(&table, &key, span)?;
    let file_path = file_path.to_string_lossy();

    Ok(quote! {
        const _: &str = include_str!(#file_path);

        #tokens
    })
}

fn codegen(table: &toml::Table, key: &str, span: Span2) -> syn::Result<TokenStream2> {
    let mut tokens = Vec::new();
    for (key, value) in table {
        tokens.push(match value {
            toml::Value::String(template) => {
                let fn_name = format_ident!("{}", key.to_snake_case());
                let placeholders = super::find_placeholders(template);
                let placeholders = super::normalize_placeholders(&placeholders);
                super::codegen_fn(template, &fn_name, &placeholders)
            }
            toml::Value::Table(table) => codegen(table, key, span)?,
            v => {
                return Err(syn::Error::new(
                    span,
                    format!("expected string or table, got {}", v.type_str())
                ))
            }
        });
    }

    let ident = format_ident!("{}", key);
    Ok(quote! {
        pub mod #ident {
            #(#tokens)*
        }
    })
}
