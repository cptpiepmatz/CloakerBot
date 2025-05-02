use std::{collections::HashMap, fs, path::{Path, PathBuf}};

use heck::ToSnakeCase;
use itertools::Itertools;
use proc_macro2::{Ident as Ident2, Span as Span2, TokenStream as TokenStream2};
use quote::{format_ident, quote};
use syn::{parse::{Parse, ParseStream}, LitStr, Token};

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
                    &format!("expected `fn_name`, got `{ident}`")
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
    let args: Args = syn::parse2(input)?;

    let content = fs::read_to_string(&args.file_path.0)
        .map_err(|err| syn::Error::new(args.file_path.1, err))?;

    let formatted = format(&content);
    super::valid(&formatted, args.file_path.1)?;

    let placeholders = super::find_placeholders(&formatted);
    let placeholders = super::normalize_placeholders(&placeholders);

    let fn_name = match args.fn_name {
        Some(fn_name) => fn_name,
        None => {
            let file_name =
                args.file_path.0.file_name().ok_or_else(|| {
                    syn::Error::new(args.file_path.1, "could not get the file name")
                })?;
            format_ident!("{}", file_name.to_string_lossy().trim_end_matches(".md").to_snake_case())
        }
    };

    Ok(codegen(&formatted, &fn_name, &placeholders, &args.file_path.0))
}

fn format(input: &str) -> String {
    let s = input.trim().lines().map(|line| line.trim()).join("\n");
    let s = s.trim();

    s.replace("\n\n\n", "<br><br>")
        .replace("\n\n", "<br>")
        .replace("\n", " ")
        .replace("  ", " ")
        .replace("<br>", "\n")
}

fn codegen(
    input: &str,
    fn_name: &Ident2,
    placeholders: &HashMap<String, String>,
    file_path: &Path
) -> TokenStream2 {
    let mut lit = input.to_owned();
    for (from, to) in placeholders.iter() {
        lit = lit.replace(&format!("{{{from}}}"), &format!("{{{to}}}"));
    }

    let imports = quote! {
        use std::borrow::Borrow;
        use twilight_mention::fmt::{MentionFormat, CommandMention};
        use twilight_model::id::{Id, marker::UserMarker};
        use std::fmt::Display;
    };

    let file_path = file_path.display().to_string();

    struct Mapping {
        sort_key: String,
        ident: Ident2,
        ty: TokenStream2,
        op: TokenStream2
    }

    let mut mappings: Vec<Mapping> = placeholders
        .iter()
        .map(|(name, ident)| {
            let ident = format_ident!("{}", ident);
            match name.as_str() {
                "CloakerBot" => Mapping {
                    sort_key: "__cloaker_bot".to_string(),
                    ident: ident.clone(),
                    ty: quote!(impl Borrow<MentionFormat<Id<UserMarker>>>),
                    op: quote!(let #ident = #ident.borrow();)
                },
                s if s.starts_with("/") => Mapping {
                    sort_key: format!("_{}", ident),
                    ident: ident.clone(),
                    ty: quote!(impl Borrow<MentionFormat<CommandMention>>),
                    op: quote!(let #ident = #ident.borrow();)
                },
                _ => Mapping {
                    sort_key: ident.to_string(),
                    ident: ident.clone(),
                    ty: quote!(impl Display),
                    op: quote!()
                }
            }
        })
        .collect();
    mappings.sort_by(|Mapping { sort_key: a, .. }, Mapping { sort_key: b, .. }| a.cmp(b));

    let args = mappings.iter().map(|Mapping { ident, ty, .. }| quote!(#ident: #ty));

    let ops = mappings.iter().map(|Mapping { op, .. }| op);

    quote! {
        mod #fn_name {
            #imports

            const _: &str = std::include_str!(#file_path);

            pub fn #fn_name(#(#args),*) -> std::string::String {
                #(#ops)*
                std::format!(#lit)
            }
        }

        pub use #fn_name::#fn_name;
    }
}