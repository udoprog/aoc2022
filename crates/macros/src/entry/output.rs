use std::ops;

use proc_macro::{Literal, Span, TokenTree};

use crate::error::Error;
use crate::into_tokens::{braced, from_fn, parens, IntoTokens};
use crate::token_stream::TokenStream;

const S: [char; 2] = [':', ':'];
const T: [char; 2] = ['=', '>'];

#[derive(Default)]
pub(crate) struct Config {
    pub(crate) input_file: Option<Literal>,
}

impl Config {
    /// Validate the parsed configuration.
    pub(crate) fn validate(&self, errors: &mut Vec<Error>) {
        if self.input_file.is_none() {
            errors.push(Error::new(Span::call_site(), "missing `input` argument"));
        }
    }
}

/// The parsed item output.
pub(crate) struct ItemOutput {
    tokens: Vec<TokenTree>,
    fn_name: Option<usize>,
    signature: Option<ops::Range<usize>>,
    block: Option<usize>,
    args: Option<usize>,
}

impl ItemOutput {
    pub(crate) fn new(
        tokens: Vec<TokenTree>,
        fn_name: Option<usize>,
        signature: Option<ops::Range<usize>>,
        block: Option<usize>,
        args: Option<usize>,
    ) -> Self {
        Self {
            tokens,
            fn_name,
            signature,
            block,
            args,
        }
    }

    /// Validate the parsed item.
    pub(crate) fn validate(&self, _: &mut Vec<Error>) {}

    pub(crate) fn block_span(&self) -> Option<Span> {
        let block = *self.block.as_ref()?;
        Some(self.tokens.get(block)?.span())
    }

    /// Expand into a function item.
    pub(crate) fn expand_item(self, config: Config) -> impl IntoTokens {
        from_fn(move |s| {
            if let Some(item) = self.expand_if_present(config) {
                s.write(item);
            } else {
                s.write(&self.tokens[..]);
            }
        })
    }

    /// Expands the function item if all prerequisites are present.
    fn expand_if_present(&self, config: Config) -> Option<impl IntoTokens + '_> {
        let m = Mod;

        let signature = self.signature.as_ref()?.clone();
        let signature = self.tokens.get(signature)?;
        let fn_name = self.tokens.get(self.fn_name?)?;
        let args = self.args?;

        let (input_decl, input_arg) = match config.input_file {
            Some(input) => {
                let decl = (("let", "input"), '=', (m, "input", '!', parens(input)), ';');

                (Some(decl), Input::Input)
            }
            None => (None, Input::Todo),
        };

        let parse_opts = (
            ("let", "opts"),
            '=',
            (m, "cli", S, "Opts", S, "parse", parens(()), '?'),
            ';',
        );

        let block = (parse_opts, input_decl);

        let signature = expand_without_index(signature, args, || parens(()));

        let mode = (m, "cli", S, "Mode");

        let call_mode = (
            (mode, S, "Default"),
            T,
            braced(ReturnCall(fn_name.clone(), input_arg)),
        );
        let bench_mode = (
            (mode, S, "Bench"),
            T,
            braced(bencher(m, BenchCall(fn_name.clone(), input_arg))),
        );

        let match_mode = (
            "match",
            ("opts", '.', "mode"),
            braced((call_mode, bench_mode)),
        );

        Some((signature, braced((&self.tokens[..], block, match_mode))))
    }
}

fn bencher(m: Mod, call: impl IntoTokens) -> impl IntoTokens {
    from_fn(move |s| {
        s.write((
            ("let", "mut", "b"),
            '=',
            (m, "cli", S, "Bencher", S, "new", parens(()), ';'),
        ));

        s.write((
            "b",
            '.',
            "iter",
            parens(('&', "opts", ',', ['|', '|'], call)),
        ));
    })
}

fn expand_without_index<'a, R: 'a, T>(
    tokens: &'a [TokenTree],
    index: usize,
    replace: R,
) -> impl IntoTokens + 'a
where
    R: Fn() -> T,
    T: IntoTokens + 'a,
{
    from_fn(move |s| {
        for (n, tt) in tokens.iter().enumerate() {
            if n == index {
                s.write(replace());
            } else {
                s.write(tt.clone());
            }
        }
    })
}

#[derive(Clone, Copy)]
struct Mod;

impl IntoTokens for Mod {
    fn into_tokens(self, stream: &mut TokenStream, span: Span) {
        stream.write(span, ("lib", S));
    }
}

#[derive(Clone, Copy)]
enum Input {
    Input,
    Todo,
}

impl IntoTokens for Input {
    fn into_tokens(self, stream: &mut TokenStream, span: Span) {
        match self {
            Input::Input => {
                stream.write(span, "input");
            }
            Input::Todo => {
                stream.write(span, "todo");
                stream.write(span, '!');
                stream.write(span, parens(()));
            }
        }
    }
}

struct ReturnCall(TokenTree, Input);

impl IntoTokens for ReturnCall {
    fn into_tokens(self, stream: &mut TokenStream, span: Span) {
        let ReturnCall(name, input) = self;
        stream.write(span, ("return", name, parens(input), ';'));
    }
}

struct BenchCall(TokenTree, Input);

impl IntoTokens for BenchCall {
    fn into_tokens(self, stream: &mut TokenStream, span: Span) {
        let BenchCall(name, input) = self;
        stream.write(span, (name, parens((input, '.', "clone", parens(())))));
    }
}
