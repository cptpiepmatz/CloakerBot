use std::path::PathBuf;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;

mod env;
mod markdown;
mod template;

fn resolve_path(file_path: &syn::LitStr) -> syn::Result<PathBuf> {
    let span = file_path.span();
    let source = span
        .unwrap()
        .local_file()
        .ok_or_else(|| syn::Error::new(span, "could not determine the macro's source file"))?;
    let source = std::path::absolute(source).map_err(|error| syn::Error::new(span, error))?;
    let directory = source
        .parent()
        .ok_or_else(|| syn::Error::new(span, "macro source file has no parent directory"))?;
    Ok(directory.join(file_path.value()))
}

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
    /// Generates a lazy display template, or formatted component templates with
    /// `components = true`.
    ///
    /// The template path is relative to the source file invoking the macro. The
    /// generated function is named after the Markdown file unless `fn_name` is
    /// specified.
    ///
    /// # Component templates
    ///
    /// Use `components = true` to generate Discord component templates rather
    /// than one Markdown display value:
    ///
    /// ```ignore
    /// macros::template_md!("update.md", fn_name = update, components = true);
    /// ```
    ///
    /// The generated function takes all placeholders used in the document,
    /// including HTML attributes and button labels. It returns an
    /// `ExactSizeIterator` of `ComponentTemplate` values with text already formatted
    /// into owned strings. Convert each value into a Twilight `Component` when
    /// preparing the message. The generated code uses
    /// the calling crate's `crate::util::component_template::ComponentTemplate`.
    ///
    /// Keep the `.md` extension. This is Markdown with a small set of component
    /// elements, not MDX or JSX. Put component elements on their own lines, with
    /// blank lines around Markdown content inside containers.
    ///
    /// ## Markdown
    ///
    /// Markdown becomes text displays, with a new text display starting at each
    /// heading or component element. Headings remain part of the displayed text.
    /// Soft line breaks become spaces; code blocks and hard line breaks are
    /// preserved. Blockquotes remain Markdown quotes.
    ///
    /// A Markdown horizontal rule (`---`) becomes a visible, small separator.
    ///
    /// ## Separators
    ///
    /// ```html
    /// <hr size="large" divider="false">
    /// ```
    ///
    /// `size` is `small` (the default) or `large`. `divider` defaults to `true`;
    /// set it to `false` for spacing without a visible line. Separators are
    /// explicit, including spacing before the final action row.
    ///
    /// ## Colored containers
    ///
    /// ```html
    /// <container color="#ffcc4d">
    ///
    /// ⚠️ Takedowns from a Danger Zone cannot get deflected.
    ///
    /// </container>
    /// ```
    ///
    /// Containers require a six-digit RGB color and 1–10 child components.
    /// Nested containers are not supported.
    ///
    /// ## Buttons
    ///
    /// ```html
    /// <action-row>
    /// <button style="secondary" custom-id="{disable_notifications}">Don't send me owner notifications</button>
    /// </action-row>
    /// ```
    ///
    /// Action rows contain 1–5 buttons. Each button requires a label and
    /// `custom-id`, which can include a placeholder. Supported styles are
    /// `primary`, `secondary` (the default), `success`, and `danger`.
    /// `disabled="true"` disables a button.
    ///
    /// Keep the action row at the end of the document to show it only in the
    /// final notification message. Message packing preserves component order
    /// and keeps containers and action rows intact.
    ///
    /// ## HTML parsing and validation
    ///
    /// HTML nesting and entity decoding are handled by `scraper`/`html5ever`.
    /// As with a browser, malformed HTML may be repaired automatically. The
    /// resulting component tree is still validated: unsupported elements,
    /// attributes, invalid values, and invalid Discord component nesting are
    /// compile-time errors. This syntax is not a general HTML renderer. Use
    /// explicit closing tags for non-void elements such as `container`,
    /// `action-row`, and `button`.
    template_md: template::md,
    template_toml: template::toml,
    env: env::env,
}
