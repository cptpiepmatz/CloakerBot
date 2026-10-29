use std::env::VarError;
use std::fmt::Display;

use proc_macro2::{Span as Span2, TokenStream as TokenStream2};
use quote::{ToTokens, quote};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{Ident, Lit, LitStr, Token};

struct Args {
    env: LitStr,
    dev: Option<Lit>,
    prod: Option<Lit>,
    fallback: Option<Lit>
}

impl Parse for Args {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let env: LitStr = input.parse()?;
        let mut dev = None;
        let mut prod = None;
        let mut fallback = None;
        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            let options: Punctuated<ArgsOption, Token![,]> =
                Punctuated::parse_separated_nonempty(input)?;
            for option in options {
                match option {
                    ArgsOption::Dev(lit) => dev = Some(lit),
                    ArgsOption::Prod(lit) => prod = Some(lit),
                    ArgsOption::Fallback(lit) => fallback = Some(lit)
                }
            }
        }
        Ok(Self { env, dev, prod, fallback })
    }

    // macros::env!("SOME_ENV", dev = "lol", prod = "kek", "other")
}

enum ArgsOption {
    Dev(Lit),
    Prod(Lit),
    Fallback(Lit)
}

impl Parse for ArgsOption {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(Lit) {
            return Ok(Self::Fallback(input.parse()?));
        }

        if input.peek(Ident) {
            let key: Ident = input.parse()?;
            input.parse::<Token![=]>()?;
            let value: Lit = input.parse()?;
            return match key.to_string().as_str() {
                "dev" => Ok(Self::Dev(value)),
                "prod" => Ok(Self::Prod(value)),
                _ => Err(syn::Error::new(key.span(), "expected `dev` or `prod`, got `{key}`"))
            };
        }

        Err(syn::Error::new(input.span(), "expected string literal or ident"))
    }
}

pub fn env(input: TokenStream2) -> syn::Result<TokenStream2> {
    let span = input.span();
    let Args { env, dev, prod, fallback } = syn::parse2(input)?;

    match (&dev, &prod, &fallback) {
        (_, _, Some(_)) => {}
        (Some(_), Some(_), _) => {}
        _ => {
            return Err(syn::Error::new(
                span,
                "non-exhaustive fallback, `fallback` is required unless both `dev` and `prod` are \
                 provided"
            ));
        }
    }

    let lits = [dev.as_ref(), prod.as_ref(), fallback.as_ref()].into_iter().filter_map(|x| x);
    let guessed_type = GuessedType::try_from_iter(lits)?;

    match std::env::var(&env.value()) {
        Err(VarError::NotPresent) => {}
        Err(err @ VarError::NotUnicode(_)) => return Err(syn::Error::new(env.span(), err)),
        Ok(value) => {
            fn map_err<E: Display>(span: Span2) -> impl Fn(E) -> syn::Error {
                move |err: E| syn::Error::new(span, err)
            }

            let value: &dyn ToTokens = match guessed_type {
                GuessedType::Bool => &value.parse::<bool>().map_err(map_err(env.span()))?,
                GuessedType::F32 => &value.parse::<f32>().map_err(map_err(env.span()))?,
                GuessedType::F64 => &value.parse::<f64>().map_err(map_err(env.span()))?,
                GuessedType::I8 => &value.parse::<i8>().map_err(map_err(env.span()))?,
                GuessedType::I16 => &value.parse::<i16>().map_err(map_err(env.span()))?,
                GuessedType::I32 => &value.parse::<i32>().map_err(map_err(env.span()))?,
                GuessedType::I64 => &value.parse::<i64>().map_err(map_err(env.span()))?,
                GuessedType::Isize => &value.parse::<isize>().map_err(map_err(env.span()))?,
                GuessedType::Str => &value.parse::<String>().map_err(map_err(env.span()))?,
                GuessedType::U8 => &value.parse::<u8>().map_err(map_err(env.span()))?,
                GuessedType::U16 => &value.parse::<u16>().map_err(map_err(env.span()))?,
                GuessedType::U32 => &value.parse::<u32>().map_err(map_err(env.span()))?,
                GuessedType::U64 => &value.parse::<u64>().map_err(map_err(env.span()))?,
                GuessedType::Usize => &value.parse::<usize>().map_err(map_err(env.span()))?
            };

            return Ok(quote! {{
                let _ = std::env!(#env);
                #value
            }});
        }
    }

    let dev = dev.map(|value| quote!(#[cfg(feature = "dev")] let value = #value;));
    let prod = prod.map(|value| quote!(#[cfg(feature = "prod")] let value = #value;));
    let fallback = fallback.map(|value| quote!(let value = #value;));

    Ok(quote! {{
        std::option_env!(#env);

        #fallback
        #prod
        #dev
        value
    }})
}

#[derive(PartialEq, Eq)]
enum GuessedType {
    Bool,
    F32,
    F64,
    I8,
    I16,
    I32,
    I64,
    Isize,
    Str,
    U8,
    U16,
    U32,
    U64,
    Usize
}

impl TryFrom<&Lit> for GuessedType {
    type Error = syn::Error;

    fn try_from(value: &Lit) -> Result<Self, Self::Error> {
        Ok(match value {
            Lit::Str(_) => GuessedType::Str,
            Lit::Int(lit_int) => match lit_int.suffix() {
                "" => return Err(syn::Error::new(value.span(), "missing int suffix")),
                "i8" => GuessedType::I8,
                "i16" => GuessedType::I16,
                "i32" => GuessedType::I32,
                "i64" => GuessedType::I64,
                "isize" => GuessedType::Isize,
                "u8" => GuessedType::U8,
                "u16" => GuessedType::U16,
                "u32" => GuessedType::U32,
                "u64" => GuessedType::U64,
                "usize" => GuessedType::Usize,
                suffix => {
                    return Err(syn::Error::new(
                        value.span(),
                        format!("unexpected int suffix {suffix}")
                    ));
                }
            },
            Lit::Float(lit_float) => match lit_float.suffix() {
                "" => return Err(syn::Error::new(value.span(), "missing float suffix")),
                "f32" => GuessedType::F32,
                "f64" => GuessedType::F64,
                suffix => {
                    return Err(syn::Error::new(
                        value.span(),
                        format!("unexpected float suffix {suffix}")
                    ));
                }
            },
            Lit::Bool(_) => GuessedType::Bool,
            _ => {
                return Err(syn::Error::new(value.span(), "guessing this type is not implemented"));
            }
        })
    }
}

impl GuessedType {
    fn try_from_iter<'i>(iter: impl IntoIterator<Item = &'i Lit>) -> syn::Result<Self> {
        let mut iter = iter.into_iter();

        let guessed_type = match iter.next() {
            None => return Ok(GuessedType::Str),
            Some(value) => GuessedType::try_from(value)?
        };

        for other in iter {
            let other_guessed_type = GuessedType::try_from(other)?;
            if other_guessed_type != guessed_type {
                return Err(syn::Error::new(other.span(), "guessed different types"));
            }
        }

        Ok(guessed_type)
    }
}
