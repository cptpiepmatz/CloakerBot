use std::{collections::HashMap, fs, path::{Path, PathBuf}};

use heck::ToSnakeCase;
use proc_macro2::{Ident as Ident2, Span as Span2, TokenStream as TokenStream2};
use quote::{format_ident, quote};
use syn::{parse::{Parse, ParseStream}, LitStr, Token};

pub fn macro_impl(input: TokenStream2) -> syn::Result<TokenStream2> {
    todo!()
}