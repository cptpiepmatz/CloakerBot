use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use heck::ToSnakeCase;
use itertools::Itertools;
use proc_macro2::{Ident as Ident2, Span, TokenStream as TokenStream2};
use quote::{format_ident, quote};
use regex::Regex;
use syn::parse::{Parse, ParseStream};
use syn::{LitStr, Token};

struct Args {
    file_path: (PathBuf, Span),
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

pub fn discord_message_template2(input: TokenStream2) -> Result<TokenStream2, syn::Error> {
    let args: Args = syn::parse2(input)?;

    let content = fs::read_to_string(&args.file_path.0)
        .map_err(|err| syn::Error::new(args.file_path.1, err))?;

    let formatted = format(&content);
    check(&formatted, args.file_path.1)?;

    let placeholders = find_placeholders(&formatted);
    let placeholders = normalize_placeholders(&placeholders);

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

fn check(input: &str, span: Span) -> Result<(), syn::Error> {
    if input.contains("{}") {
        return Err(syn::Error::new(span, "{} not allowed in content"));
    }

    Ok(())
}

fn find_placeholders(template: &str) -> HashSet<String> {
    let re = Regex::new(r"\{([^}]+)\}").expect("valid regex"); // Match content inside { }
    let mut placeholders = HashSet::new();

    for cap in re.captures_iter(template) {
        if let Some(matched) = cap.get(1) {
            placeholders.insert(matched.as_str().to_string());
        }
    }

    placeholders
}

fn normalize_placeholders(placeholders: &HashSet<String>) -> HashMap<String, String> {
    let mut map = HashMap::new();

    for placeholder in placeholders {
        map.insert(placeholder.to_owned(), placeholder.to_snake_case());
    }

    map
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

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    #[test]
    fn find_placeholders() {
        let haystack = "{CloakerBot} {something} {/some command}";
        let placeholders = super::find_placeholders(haystack);
        assert_eq!(placeholders.len(), 3);
        assert!(placeholders.contains("CloakerBot"));
        assert!(placeholders.contains("something"));
        assert!(placeholders.contains("/some command"));
    }

    #[test]
    fn normalize_placeholders() {
        let placeholders = super::normalize_placeholders(&HashSet::from_iter(
            ["CloakerBot", "something", "/some command"].map(ToOwned::to_owned)
        ));
        assert_eq!(placeholders.get("CloakerBot").unwrap(), "cloaker_bot");
        assert_eq!(placeholders.get("something").unwrap(), "something");
        assert_eq!(placeholders.get("/some command").unwrap(), "some_command");
    }
}
