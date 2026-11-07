use std::collections::{HashMap, HashSet};

use heck::{ToShoutySnakeCase, ToSnakeCase};
use proc_macro2::{Ident as Ident2, Span as Span2, TokenStream as TokenStream2};
use quote::{format_ident, quote};
use regex::Regex;

mod md;
pub use md::macro_impl as md;

mod components;

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
    codegen(&[input], fn_name, placeholders, None)
}

fn codegen_components(
    components: &[components::Node],
    fn_name: &Ident2,
    span: Span2
) -> syn::Result<TokenStream2> {
    let mut templates = Vec::new();
    let components: Vec<_> =
        components.iter().map(|component| component.codegen(&mut templates)).collect();
    let mut placeholders = HashSet::new();
    for template in &templates {
        valid(template, span)?;
        placeholders.extend(find_placeholders(template));
    }
    let placeholders = normalize_placeholders(&placeholders);
    let templates: Vec<_> = templates.iter().map(String::as_str).collect();
    Ok(codegen(&templates, fn_name, &placeholders, Some(&components)))
}

fn codegen(
    input: &[&str],
    fn_name: &Ident2,
    placeholders: &HashMap<String, String>,
    components: Option<&[TokenStream2]>
) -> TokenStream2 {
    let literals: Vec<_> = input
        .iter()
        .map(|input| {
            let mut lit = (*input).to_owned();
            for (from, to) in placeholders {
                lit = lit.replace(&format!("{{{from}}}"), &format!("{{{to}}}"));
            }
            lit
        })
        .collect();

    let imports = quote! {
        use std::borrow::Borrow;
        use twilight_mention::fmt::{MentionFormat, CommandMention};
        use twilight_model::id::{Id, marker::UserMarker};
        use std::fmt::Display;
        use std::convert::AsRef;
        use crate::util::template_format::{Template, TemplateFormat};
    };

    struct Mapping {
        sort_key: String,
        ident: Ident2,
        ty: TokenStream2,
        op: TokenStream2,
        generic: Option<TokenStream2>,
        constraints: Vec<TokenStream2>
    }

    let mut mappings: Vec<Mapping> = placeholders
        .iter()
        .map(|(name, ident)| {
            let generic = format_ident!("{}", ident.to_shouty_snake_case());
            let ident = format_ident!("{}", ident);
            match name.as_str() {
                "CloakerBot" => Mapping {
                    sort_key: "____cloaker_bot".to_string(),
                    ident: ident.clone(),
                    ty: quote!(impl Borrow<MentionFormat<Id<UserMarker>>>),
                    op: quote!(let #ident = #ident.borrow();),
                    generic: None,
                    constraints: vec![]
                },
                s if s.starts_with('/') => Mapping {
                    sort_key: format!("___{}", ident),
                    ident: ident.clone(),
                    ty: quote!(impl Borrow<MentionFormat<CommandMention>>),
                    op: quote!(let #ident = #ident.borrow();),
                    generic: None,
                    constraints: vec![]
                },
                s if s.starts_with('#') => Mapping {
                    sort_key: format!("__{}", ident),
                    ident: ident.clone(),
                    ty: quote!(#generic),
                    op: quote!(let #ident = #ident.template();),
                    generic: Some(quote!(#generic)),
                    constraints: vec![
                        quote!(#generic: Template),
                        quote!(for<'t> TemplateFormat<'t, #generic>: Display),
                    ]
                },
                "quote" => Mapping {
                    // last every time
                    sort_key: "quote".to_string(),
                    ident: ident.clone(),
                    ty: quote!(impl AsRef<str>),
                    op: quote!(let #ident = #ident.as_ref();),
                    generic: None,
                    constraints: vec![]
                },
                _ => Mapping {
                    sort_key: format!("_{}", ident),
                    ident: ident.clone(),
                    ty: quote!(impl Display),
                    op: quote!(),
                    generic: None,
                    constraints: vec![]
                }
            }
        })
        .collect();
    mappings.sort_by(|Mapping { sort_key: a, .. }, Mapping { sort_key: b, .. }| a.cmp(b));

    let args = mappings.iter().map(|Mapping { ident, ty, .. }| quote!(#ident: #ty));

    let ops = mappings.iter().map(|Mapping { op, .. }| op);

    let (return_type, body) = if let Some(components) = components {
        let indices = 0..literals.len();
        let text = if mappings.is_empty() {
            quote! {
                let text = |index: usize| {
                    match index {
                        #(#indices => #literals.to_owned(),)*
                        _ => unreachable!()
                    }
                };
            }
        }
        else {
            quote! {
                #(#ops)*
                let text = |index: usize| {
                    match index {
                        #(#indices => std::format!(#literals),)*
                        _ => unreachable!()
                    }
                };
            }
        };
        let body = quote! {
            use crate::util::component_template::ComponentTemplate as Component;
            use twilight_model::channel::message::component::{ButtonStyle, SeparatorSpacingSize};
            #text
            let _ = &text;
            let components: Vec<Component> = vec![#(#components),*];
            components.into_iter()
        };
        (
            quote!(
                impl ExactSizeIterator<Item = crate::util::component_template::ComponentTemplate>
            ),
            body
        )
    }
    else {
        let lit = &literals[0];
        let body = if mappings.is_empty() {
            quote!(#lit)
        }
        else {
            quote! {
                std::fmt::from_fn(move |f| {
                    #(#ops)*
                    std::write!(f, #lit)
                })
            }
        };
        (quote!(impl Display), body)
    };

    let generics = mappings.iter().filter_map(|Mapping { generic, .. }| generic.as_ref());

    let constraints = mappings.iter().flat_map(|Mapping { constraints, .. }| constraints);

    quote! {
        mod #fn_name {
            #imports

            pub fn #fn_name<#(#generics),*>(#(#args),*) -> #return_type
                where #(#constraints),*
            {
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
