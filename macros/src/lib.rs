use proc_macro::TokenStream;
use proc_macro_error2::{Diagnostic, proc_macro_error};
use proc_macro2::TokenStream as TokenStream2;

mod template;

#[proc_macro]
#[proc_macro_error]
pub fn template_md(input: TokenStream) -> TokenStream {
    defer(input, template::md)
}

#[proc_macro]
#[proc_macro_error]
pub fn template_toml(input: TokenStream) -> TokenStream {
    defer(input, template::toml)
}

fn defer(input: TokenStream, f: impl Fn(TokenStream2) -> syn::Result<TokenStream2>) -> TokenStream {
    let input = TokenStream2::from(input);
    let output = match f(input) {
        Ok(output) => output,
        Err(err) => Diagnostic::from(err).abort()
    };
    TokenStream::from(output)
}
