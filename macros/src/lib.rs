use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;

mod env;
mod split_md;
mod template;

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
