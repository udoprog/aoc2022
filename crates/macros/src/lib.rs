use proc_macro::TokenStream;

mod entry;
mod error;
mod into_tokens;
mod parsing;
mod token_stream;

#[proc_macro_attribute]
pub fn entry(args: TokenStream, item_stream: TokenStream) -> TokenStream {
    crate::entry::build(args, item_stream)
}
