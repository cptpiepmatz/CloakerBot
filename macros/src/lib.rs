use discord_message_template::discord_message_template2;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use proc_macro_error2::{proc_macro_error, Diagnostic};

mod discord_message_template;

#[proc_macro]
#[proc_macro_error]
pub fn discord_message_template(input: TokenStream) -> TokenStream {
    let input = TokenStream2::from(input);
    let output = match discord_message_template2(input) {
        Ok(output) => output,
        Err(err) => Diagnostic::from(err).abort()
    };
    TokenStream::from(output)
}
