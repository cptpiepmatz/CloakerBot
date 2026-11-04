use std::path::PathBuf;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;

mod env;
mod markdown;
mod split_md;
mod template;

fn resolve_path(file_path: &syn::LitStr) -> syn::Result<PathBuf> {
    let span = file_path.span();
    let source = span
        .unwrap()
        .local_file()
        .ok_or_else(|| syn::Error::new(span, "could not determine the macro's source file"))?;
    let source = std::path::absolute(source).map_err(|error| syn::Error::new(span, error))?;
    let directory = source
        .parent()
        .ok_or_else(|| syn::Error::new(span, "macro source file has no parent directory"))?;
    Ok(directory.join(file_path.value()))
}

fn defer(input: TokenStream, f: impl Fn(TokenStream2) -> syn::Result<TokenStream2>) -> TokenStream {
    let input = TokenStream2::from(input);
    let output = match f(input) {
        Ok(output) => output,
        Err(err) => err.into_compile_error()
    };
    TokenStream::from(output)
}

macro_rules! defer_macros {
    ($($(#[$meta:meta])* $ident:ident: $f:expr),* $(,)?) => {$(
        $(#[$meta])*
        #[proc_macro]
        pub fn $ident(input: TokenStream) -> TokenStream {
            defer(input, $f)
        }
    )*}
}

defer_macros! {
    template_md: template::md,
    template_toml: template::toml,
    env: env::env,

    /// Splits a Markdown file into a static slice of
    /// `crate::assets::MarkdownSection`.
    split_md: split_md::macro_impl,
}
