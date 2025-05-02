use std::collections::{HashMap, HashSet};
use std::path::Path;

use heck::ToSnakeCase;
use proc_macro2::{Ident as Ident2, Span as Span2, TokenStream as TokenStream2};
use quote::{format_ident, quote};
use regex::Regex;

mod md;
pub use md::macro_impl as md;

mod toml;
pub use toml::macro_impl as toml;

fn valid(input: &str, span: Span2) -> syn::Result<()> {
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

fn codegen_fn(
    input: &str,
    fn_name: &Ident2,
    placeholders: &HashMap<String, String>
) -> TokenStream2 {
    let mut lit = input.to_owned();
    for (from, to) in placeholders.iter() {
        lit = lit.replace(&format!("{{{from}}}"), &format!("{{{to}}}"));
    }

    let imports = quote! {
        use std::borrow::{Borrow, Cow};
        use twilight_mention::fmt::{MentionFormat, CommandMention};
        use twilight_model::id::{Id, marker::UserMarker};
        use std::fmt::Display;
    };

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

    let body = match args.len() {
        0 => quote!(Cow::Borrowed(#lit)),
        _ => quote! {
            #(#ops)*
            Cow::Owned(std::format!(#lit))
        }
    };

    quote! {
        mod #fn_name {
            #imports

            pub fn #fn_name(#(#args),*) -> Cow<'static, str> {
                #body
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
