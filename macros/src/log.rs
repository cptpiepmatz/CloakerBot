use common::log::LogLevel;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{parse::{Parse, ParseStream}, punctuated::Punctuated, Expr, LitStr, Token};

struct Args {
    pattern: LitStr,
    items: Vec<Expr>
}

impl Parse for Args {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let pattern: LitStr = input.parse()?;
        let items = match input.peek(Token![,]) {
            false => vec![],
            true => {
                input.parse::<Token![,]>()?;
                let items: Punctuated<Expr, Token![,]> = Punctuated::parse_separated_nonempty(input)?;
                items.into_iter().collect()
            }
        };

        Ok(Self{pattern, items})
    }
}

pub fn logger(name: &str, level: LogLevel, color: &str) -> impl Fn(TokenStream2) -> syn::Result<TokenStream2> {
    move |input: TokenStream2| {
        let Args {pattern, items} = syn::parse2(input)?;

        let level = match level {
            LogLevel::Debug => format_ident!("Debug"),
            LogLevel::Info => format_ident!("Info"),
            LogLevel::Warn => format_ident!("Warn"),
            LogLevel::Error => format_ident!("Error"),
        };

        Ok(quote! {{
            use std::io::IsTerminal;

            if common::log::log_level() <= common::log::LogLevel::#level {
                let timestamp = chrono::offset::Utc::now();
                match std::io::stdout().is_terminal() {
                    true => paris::log!(
                        "<{}> [<{}>{}</>] {}",
                        timestamp.format("%H:%M:%S"),
                        #color,
                        #name,
                        std::format_args!(#pattern #(, #items)*)
                    ),
                    false => paris::log!(
                        "<{}> [{}] {}",
                        timestamp.format("%Y-%m-%d %H:%M:%S"),
                        #name,
                        std::format_args!(#pattern #(, #items)*)
                    )
                }
            }
        }})
    }
}