use proc_macro::{Delimiter, Ident, Literal, Punct, Spacing, Span, TokenTree};

use crate::token_stream::TokenStream;

pub(crate) trait IntoTokens {
    /// Convert into tokens.
    fn into_tokens(self, stream: &mut TokenStream, span: Span);
}

impl IntoTokens for TokenStream {
    #[inline]
    fn into_tokens(self, stream: &mut TokenStream, _: Span) {
        stream.extend(self);
    }
}

impl IntoTokens for proc_macro::TokenStream {
    fn into_tokens(self, stream: &mut TokenStream, _: Span) {
        for tt in self {
            stream.push(tt);
        }
    }
}

impl IntoTokens for Literal {
    #[inline]
    fn into_tokens(self, stream: &mut TokenStream, _: Span) {
        let tt = TokenTree::Literal(self);
        stream.push(tt);
    }
}

impl IntoTokens for TokenTree {
    #[inline]
    fn into_tokens(self, stream: &mut TokenStream, _: Span) {
        stream.push(self);
    }
}

impl<T> IntoTokens for Option<T>
where
    T: IntoTokens,
{
    #[inline]
    fn into_tokens(self, stream: &mut TokenStream, span: Span) {
        if let Some(tt) = self {
            tt.into_tokens(stream, span);
        }
    }
}

impl IntoTokens for &str {
    fn into_tokens(self, stream: &mut TokenStream, span: Span) {
        let mut ident = Ident::new(self, span);
        ident.set_span(span);
        stream.push(TokenTree::Ident(ident));
    }
}

impl<const N: usize> IntoTokens for [char; N] {
    fn into_tokens(self, stream: &mut TokenStream, span: Span) {
        let mut it = self.into_iter();

        if let Some(last) = it.next_back() {
            for c in it {
                let mut punct = Punct::new(c, Spacing::Joint);
                punct.set_span(span);
                stream.push(TokenTree::Punct(punct));
            }

            let mut punct = Punct::new(last, Spacing::Alone);
            punct.set_span(span);
            stream.push(TokenTree::Punct(punct));
        }
    }
}

impl IntoTokens for char {
    fn into_tokens(self, stream: &mut TokenStream, span: Span) {
        let mut punct = Punct::new(self, Spacing::Alone);
        punct.set_span(span);
        stream.push(TokenTree::Punct(punct));
    }
}

impl IntoTokens for usize {
    fn into_tokens(self, stream: &mut TokenStream, span: Span) {
        let mut literal = Literal::usize_unsuffixed(self);
        literal.set_span(span);
        stream.push(TokenTree::Literal(literal));
    }
}

impl IntoTokens for () {
    fn into_tokens(self, _: &mut TokenStream, _: Span) {}
}

macro_rules! tuple {
    ($($gen:ident $var:ident),*) => {
        impl<$($gen,)*> IntoTokens for ($($gen,)*) where $($gen: IntoTokens),* {
            fn into_tokens(self, stream: &mut TokenStream, span: Span) {
                let ($($var,)*) = self;
                $($var.into_tokens(stream, span);)*
            }
        }
    }
}

tuple!(A a);
tuple!(A a, B b);
tuple!(A a, B b, C c);
tuple!(A a, B b, C c, D d);
tuple!(A a, B b, C c, D d, E e);
tuple!(A a, B b, C c, D d, E e, F f);
tuple!(A a, B b, C c, D d, E e, F f, G g);
tuple!(A a, B b, C c, D d, E e, F f, G g, H h);
tuple!(A a, B b, C c, D d, E e, F f, G g, H h, I i);

struct Group<T>(Delimiter, T);

impl<T> IntoTokens for Group<T>
where
    T: IntoTokens,
{
    fn into_tokens(self, stream: &mut TokenStream, span: Span) {
        let checkpoint = stream.checkpoint();
        self.1.into_tokens(stream, span);
        stream.group(span, self.0, checkpoint);
    }
}

/// Construct a parenthesized group `(<inner>)`.
pub(crate) fn parens<T>(inner: T) -> impl IntoTokens
where
    T: IntoTokens,
{
    Group(Delimiter::Parenthesis, inner)
}

/// Construct a braced group `{<inner>}`.
pub(crate) fn braced<T>(inner: T) -> impl IntoTokens
where
    T: IntoTokens,
{
    Group(Delimiter::Brace, inner)
}

#[derive(Clone)]
pub(crate) struct FromFn<T>(T);

impl<T> IntoTokens for FromFn<T>
where
    T: FnOnce(&mut SpannedStream<'_>),
{
    fn into_tokens(self, stream: &mut TokenStream, span: Span) {
        let mut stream = SpannedStream { stream, span };
        (self.0)(&mut stream);
    }
}

/// Construct a [`IntoTokens`] implementation from a callback function.
pub(crate) fn from_fn<T>(f: T) -> FromFn<T>
where
    T: FnOnce(&mut SpannedStream<'_>),
{
    FromFn(f)
}

impl<T> Copy for FromFn<T> where T: Copy {}

/// A stream that has an implicit span associated with it.
pub(crate) struct SpannedStream<'a> {
    stream: &'a mut TokenStream,
    span: Span,
}

impl SpannedStream<'_> {
    /// Push the given sequence of tokens.
    pub(crate) fn write(&mut self, tt: impl IntoTokens) {
        self.stream.write(self.span, tt);
    }
}
