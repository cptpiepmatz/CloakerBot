use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, quote};
use std::{env::VarError, fmt::Display, str::FromStr};
use syn::{LitStr, Token, parse::Parse};

pub fn from_str<T, I>(input: TokenStream2) -> syn::Result<TokenStream2>
where
    T: FromStr,
    T: Into<I>,
    I: ToTokens,
    T::Err: Display,
{
    let input: LitStr = syn::parse2(input)?;
    let value = std::env::var(input.value()).map_err(|err| syn::Error::new(input.span(), err))?;
    let value: T = value.parse().map_err(|err| syn::Error::new(input.span(), err))?;
    let value = value.into();

    Ok(quote! {{
        const _: &'static str = std::env!(#input);

        #value
    }})
}

struct OrArgs {
    env: LitStr,
    or: LitStr,
}

impl Parse for OrArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let env: LitStr = input.parse()?;
        input.parse::<Token![,]>()?;
        let or: LitStr = input.parse()?;
        Ok(Self { env, or })
    }
}

pub fn or(input: TokenStream2) -> syn::Result<TokenStream2> {
    let OrArgs {env, or} = syn::parse2(input)?;

    match std::env::var(env.value()) {
        Ok(env) => Ok(quote!(#env)),
        Err(VarError::NotPresent) => Ok(quote!(#or)),
        Err(VarError::NotUnicode(_)) => Err(syn::Error::new(env.span(), "not unicode"))
    }
}
