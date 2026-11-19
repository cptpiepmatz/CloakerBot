use proc_macro2::{TokenStream as TokenStream2, TokenTree};
use quote::quote;
use syn::Expr;

pub fn require(input: TokenStream2) -> syn::Result<TokenStream2> {
    let mut condition = TokenStream2::new();
    let mut fallback = None;

    for token in input {
        if matches!(&token, TokenTree::Ident(ident) if ident == "else") {
            if fallback.is_some() {
                return Err(syn::Error::new(token.span(), "expected at most one `else`"));
            }
            fallback = Some(TokenStream2::new());
        }
        else {
            fallback.as_mut().unwrap_or(&mut condition).extend([token]);
        }
    }

    let condition: Expr = syn::parse2(condition)?;
    let fallback = fallback.map(syn::parse2::<Expr>).transpose()?;
    let return_value = match fallback {
        Some(value) => quote!(return #value),
        None => quote!(return)
    };

    if let Expr::Call(call) = &condition &&
        let Expr::Path(path) = call.func.as_ref() &&
        (path.path.is_ident("Some") || path.path.is_ident("Ok"))
    {
        if call.args.len() != 1 {
            return Err(syn::Error::new_spanned(call, "expected exactly one expression to unwrap"));
        }

        let value = &call.args[0];
        return if path.path.is_ident("Some") {
            Ok(quote! {
                match #value {
                    ::std::option::Option::Some(inner) => inner,
                    ::std::option::Option::None => #return_value,
                }
            })
        }
        else {
            Ok(quote! {
                match #value {
                    ::std::result::Result::Ok(inner) => inner,
                    ::std::result::Result::Err(_) => #return_value,
                }
            })
        };
    }

    Ok(quote! {
        if !(#condition) {
            #return_value;
        }
    })
}
