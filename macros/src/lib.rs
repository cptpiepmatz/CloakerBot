use proc_macro::TokenStream;
use proc_macro_error2::{Diagnostic, proc_macro_error};
use proc_macro2::TokenStream as TokenStream2;

mod env;
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

macro_rules! from_str {
    ($($name:ident < $ty:path $(, $into_ty:path)? >),*) => {$(
        #[proc_macro]
        #[proc_macro_error]
        pub fn $name(input: TokenStream) -> TokenStream {
            #[allow(dead_code)]
            type IntoType = $ty;

            {
                $(type IntoType = $into_ty;)?
                defer(input, env::from_str::<$ty, IntoType>)
            }
        }
    )*};
}

from_str! {
    env_u64<u64>,
    env_i64<i64>,
    env_u32<u32>,
    env_i32<i64>,
    env_usize<usize>,
    env_isize<isize>,
    env_non_zero_u64<std::num::NonZero<u64>, u64>,
    env_non_zero_i64<std::num::NonZero<i64>, i64>,
    env_non_zero_u32<std::num::NonZero<u32>, u32>,
    env_non_zero_i32<std::num::NonZero<i32>, i32>
}

#[proc_macro]
#[proc_macro_error]
pub fn env_or(input: TokenStream) -> TokenStream {
    defer(input, env::or)
}

fn defer(input: TokenStream, f: impl Fn(TokenStream2) -> syn::Result<TokenStream2>) -> TokenStream {
    let input = TokenStream2::from(input);
    let output = match f(input) {
        Ok(output) => output,
        Err(err) => Diagnostic::from(err).abort()
    };
    TokenStream::from(output)
}
