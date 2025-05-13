use common::log::LogLevel;
use proc_macro::TokenStream;
use proc_macro_error2::{Diagnostic, proc_macro_error};
use proc_macro2::TokenStream as TokenStream2;

mod env;
mod template;
mod log;

fn defer(input: TokenStream, f: impl Fn(TokenStream2) -> syn::Result<TokenStream2>) -> TokenStream {
    let input = TokenStream2::from(input);
    let output = match f(input) {
        Ok(output) => output,
        Err(err) => Diagnostic::from(err).abort()
    };
    TokenStream::from(output)
}

macro_rules! defer_macro {
    ($($ident:ident: $f:expr),* $(,)?) => {$(
        #[proc_macro]
        #[proc_macro_error]
        pub fn $ident(input: TokenStream) -> TokenStream {
            defer(input, $f)
        }
    )*}
}

defer_macro! {
    template_md: template::md,
    template_toml: template::toml,
    env: env::env,
    debug: log::logger("DEBUG", LogLevel::Debug, "cyan"),
    log: log::logger("LOG", LogLevel::Info, "blue"),
    info: log::logger("INFO", LogLevel::Info, "green"),
    warn: log::logger("WARN", LogLevel::Warn, "yellow"),
    error: log::logger("ERROR", LogLevel::Error, "red"),
}
