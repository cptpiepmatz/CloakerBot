use std::fs;
use std::path::PathBuf;

use heck::ToSnakeCase;
use proc_macro2::{Ident as Ident2, Span as Span2, TokenStream as TokenStream2};
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::{LitBool, LitStr, Token};

struct Args {
    file_path: (PathBuf, Span2),
    fn_name: Option<Ident2>,
    split_sections: bool
}

impl Parse for Args {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let file_path: LitStr = input.parse()?;
        let span = file_path.span();
        let file_path = crate::resolve_path(&file_path)?;

        let mut fn_name = None;
        let mut split_sections = None;
        while !input.is_empty() {
            input.parse::<Token![,]>()?;
            if input.is_empty() {
                break;
            }
            let ident: Ident2 = input.parse()?;
            input.parse::<Token![=]>()?;
            match ident.to_string().as_str() {
                "fn_name" if fn_name.is_none() => fn_name = Some(input.parse::<Ident2>()?),
                "split_sections" if split_sections.is_none() => {
                    split_sections = Some(input.parse::<LitBool>()?.value);
                }
                "fn_name" | "split_sections" => {
                    return Err(syn::Error::new(ident.span(), "duplicate option"));
                }
                _ => {
                    return Err(syn::Error::new(
                        ident.span(),
                        format!("expected `fn_name` or `split_sections`, got `{ident}`")
                    ));
                }
            }
        }

        Ok(Args {
            file_path: (file_path, span),
            fn_name,
            split_sections: split_sections.unwrap_or(false)
        })
    }
}

pub fn macro_impl(input: TokenStream2) -> syn::Result<TokenStream2> {
    let Args { file_path: (file_path, span), fn_name, split_sections } = syn::parse2(input)?;

    let content = fs::read_to_string(&file_path).map_err(|err| syn::Error::new(span, err))?;

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
    let tokens = if split_sections {
        let sections: Vec<_> = super::sections::split(&content)
            .into_iter()
            .map(|section| {
                if section.heading_level == 0 {
                    section.content
                }
                else {
                    format!(
                        "{} {}\n\n{}",
                        "#".repeat(section.heading_level.into()),
                        section.title,
                        section.content
                    )
                    .trim_end()
                    .to_owned()
                }
            })
            .collect();
        let mut placeholders = std::collections::HashSet::new();
        for section in &sections {
            super::valid(section, span)?;
            placeholders.extend(super::find_placeholders(section));
        }
        let placeholders = super::normalize_placeholders(&placeholders);
        super::codegen_sections(&sections, &fn_name, &placeholders)
    }
    else {
        let formatted = crate::markdown::format(&content);
        super::valid(&formatted, span)?;
        let placeholders = super::normalize_placeholders(&super::find_placeholders(&formatted));
        super::codegen_fn(&formatted, &fn_name, &placeholders)
    };
    Ok(quote! {
        const _: &str = std::include_str!(#file_path);

        #tokens
    })
}
