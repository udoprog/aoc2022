use core::fmt;
use core::iter::once;

use proc_macro::{Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenTree};

use crate::into_tokens::IntoTokens;
use crate::token_stream::TokenStream;

/// An error that can be raised during parsing which is associated with a span.
#[derive(Debug)]
pub(crate) struct Error {
    span: Span,
    message: Box<str>,
}

impl Error {
    pub(crate) fn new(span: Span, message: impl fmt::Display) -> Self {
        Self {
            span,
            message: message.to_string().into(),
        }
    }
}

impl IntoTokens for Error {
    fn into_tokens(self, stream: &mut TokenStream, _: Span) {
        stream.push(TokenTree::Ident(Ident::new("compile_error", self.span)));
        let mut exclamation = Punct::new('!', Spacing::Alone);
        exclamation.set_span(self.span);
        stream.push(TokenTree::Punct(exclamation));

        let mut message = Literal::string(self.message.as_ref());
        message.set_span(self.span);

        let message = once(TokenTree::Literal(message)).collect();
        let mut group = Group::new(Delimiter::Brace, message);
        group.set_span(self.span);

        stream.push(TokenTree::Group(group));
    }
}

/// Expand a message as an error.
#[cfg(feature = "tokio-entry")]
pub(crate) fn expand(message: &str) -> proc_macro::TokenStream {
    let error = Error::new(Span::call_site(), message);
    let mut stream = TokenStream::default();
    error.into_tokens(&mut stream, Span::call_site());
    stream.into_token_stream()
}
