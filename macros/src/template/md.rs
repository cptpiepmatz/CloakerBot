use std::fs;
use std::path::PathBuf;

use heck::ToSnakeCase;
use itertools::Itertools;
use proc_macro2::{Ident as Ident2, Span as Span2, TokenStream as TokenStream2};
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::{LitStr, Token};

struct Args {
    file_path: (PathBuf, Span2),
    fn_name: Option<Ident2>
}

impl Parse for Args {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let file_path: LitStr = input.parse()?;
        let span = file_path.span();
        let file_path = file_path.value();

        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("set in macro");
        let file_path = PathBuf::new().join(manifest_dir).join(file_path);

        let fn_name = if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            let ident: Ident2 = input.parse()?;
            if ident != "fn_name" {
                return Err(syn::Error::new(
                    ident.span(),
                    format!("expected `fn_name`, got `{ident}`")
                ));
            }
            input.parse::<Token![=]>()?;
            Some(input.parse::<Ident2>()?)
        }
        else {
            None
        };

        Ok(Args { file_path: (file_path, span), fn_name })
    }
}

pub fn macro_impl(input: TokenStream2) -> syn::Result<TokenStream2> {
    let Args { file_path: (file_path, span), fn_name } = syn::parse2(input)?;

    let content = fs::read_to_string(&file_path).map_err(|err| syn::Error::new(span, err))?;

    let formatted = format(&content);
    super::valid(&formatted, span)?;

    let placeholders = super::find_placeholders(&formatted);
    let placeholders = super::normalize_placeholders(&placeholders);

    let fn_name = match fn_name {
        Some(fn_name) => fn_name,
        None => {
            let file_name = file_path
                .file_name()
                .ok_or_else(|| syn::Error::new(span, "could not get the file name"))?;
            format_ident!("{}", file_name.to_string_lossy().trim_end_matches(".md").to_snake_case())
        }
    };

    let file_path = file_path.to_string_lossy();
    let tokens = super::codegen_fn(&formatted, &fn_name, &placeholders);
    Ok(quote! {
        const _: &str = std::include_str!(#file_path);

        #tokens
    })
}

pub(crate) fn format(input: &str) -> String {
    let s = input.trim().lines().map(|line| line.trim()).join("\n");
    let s = s.trim();

    s.replace("\n\n\n", "<br><br>")
        .replace("\n\n", "<br>")
        .replace("\n", " ")
        .replace("  ", " ")
        .replace("<br>", "\n")
}
