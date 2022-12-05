use proc_macro::{Delimiter, Group, Literal, Span, TokenTree};

use crate::error::Error;
use crate::into_tokens::{braced, from_fn, parens, IntoTokens};
use crate::token_stream::TokenStream;

const S: [char; 2] = [':', ':'];
const T: [char; 2] = ['=', '>'];

#[derive(Default)]
pub(crate) struct Config {
    pub(crate) input_file: Option<Literal>,
    pub(crate) expect: Option<TokenTree>,
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
}

impl ItemOutput {
    pub(crate) fn new(tokens: Vec<TokenTree>, fn_name: Option<usize>) -> Self {
        Self { tokens, fn_name }
    }

    /// Expand into a function item.
    pub(crate) fn expand_item(
        self,
        config: &Config,
        item_stream: proc_macro::TokenStream,
    ) -> impl IntoTokens + '_ {
        from_fn(move |s| {
            let fn_name = self.fn_name.and_then(|n| self.tokens.get(n));

            if let Some(fn_name) = fn_name {
                s.write(self.expand_main(config, fn_name, item_stream));
            } else {
                s.write(TokenTree::Group(Group::new(Delimiter::None, item_stream)));
            }
        })
    }

    /// Expands the function item if all prerequisites are present.
    fn expand_main<'a>(
        &'a self,
        config: &'a Config,
        fn_name: &TokenTree,
        item_stream: proc_macro::TokenStream,
    ) -> impl IntoTokens + 'a {
        let m = Mod;

        let (input_decl, input_arg) = match &config.input_file {
            Some(input) => {
                let decl = (
                    ("let", parens(("input", ',', "path"))),
                    '=',
                    (m, "input", '!', parens(input.clone())),
                    ';',
                );
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

        let mode = (m, "cli", S, "Mode");

        let compare = match &config.expect {
            Some(expect) => Compare::Expected(m, expect),
            _ => Compare::Ignore,
        };

        let call_mode = (
            (mode, S, "Default"),
            T,
            braced(CollectCall(fn_name.clone(), input_arg, m, compare)),
        );

        let bench_mode = (
            (mode, S, "Bench"),
            T,
            braced(bencher(m, BenchCall(fn_name.clone(), input_arg), compare)),
        );

        let match_mode = (
            "match",
            ("opts", '.', "mode"),
            braced((call_mode, bench_mode)),
            ';',
        );

        let ok_return = ("Ok", parens(parens(())));

        let anyhow_result = ("lib", S, "prelude", S, "Result", '<', parens(()), '>');
        let signature = ("fn", "main", parens(()), ['-', '>'], anyhow_result);

        let original = TokenTree::Group(Group::new(Delimiter::None, item_stream));
        (signature, braced((original, block, match_mode, ok_return)))
    }
}

fn bencher<'a, C: 'a>(m: Mod, call: C, compare: Compare<'a>) -> impl IntoTokens + 'a
where
    C: IntoTokens + 'a,
{
    from_fn(move |s| {
        s.write((
            ("let", "mut", "b"),
            '=',
            (m, "cli", S, "Bencher", S, "new", parens(()), ';'),
        ));

        s.write((
            ("b", '.', "iter"),
            parens((
                '&',
                "opts",
                ',',
                compare.expect(),
                ',',
                ['|', '|'],
                braced(call),
            )),
            ('?', ';'),
        ));
    })
}

#[derive(Debug, Clone, Copy)]
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

struct CollectCall<'a>(TokenTree, Input, Mod, Compare<'a>);

impl IntoTokens for CollectCall<'_> {
    fn into_tokens(self, stream: &mut TokenStream, span: Span) {
        let CollectCall(name, input, m, compare) = self;

        let handle_error = from_fn(|s| {
            let error = (
                m,
                ("cli", S, "CliError", S, "cli"),
                parens(("path", ',', "input", ',', "error")),
            );
            s.write(("return", "Err"));
            s.write(parens((error, '.', "into", parens(()))));
        });

        let handle_error = from_fn(move |s| {
            s.write("match");
            s.write((name, parens(input)));
            s.write(braced(from_fn(move |s| {
                s.write(("Ok", parens("value"), T, "value", ','));
                s.write(("Err", parens("error"), T, handle_error, ','));
            })));
        });

        stream.write(span, ("let", compare.binding(), '=', handle_error, ';'));

        stream.write(span, compare);
    }
}

struct BenchCall(TokenTree, Input);

impl IntoTokens for BenchCall {
    fn into_tokens(self, stream: &mut TokenStream, span: Span) {
        let BenchCall(name, input) = self;
        stream.write(span, (name, parens((input, '.', "clone", parens(())))));
    }
}

#[derive(Debug, Clone, Copy)]
enum Compare<'a> {
    Ignore,
    Expected(Mod, &'a TokenTree),
}

impl<'a> Compare<'a> {
    fn binding(self) -> &'static str {
        match self {
            Compare::Ignore => "_",
            Compare::Expected(..) => "value",
        }
    }

    fn expect(self) -> CompareExpect<'a> {
        match self {
            Compare::Ignore => CompareExpect::Ignore,
            Compare::Expected(_, value) => CompareExpect::Expected(value),
        }
    }
}

impl IntoTokens for Compare<'_> {
    fn into_tokens(self, stream: &mut TokenStream, _: Span) {
        if let Compare::Expected(m, tt) = self {
            let message = TokenTree::Literal(Literal::string("{:?} (value) != {:?} (expected)"));

            stream.write(tt.span(), ("let", "expected", '=', tt.clone(), ';'));

            let output_eq = (m, "cli", S, "OutputEq");

            stream.write(
                tt.span(),
                (
                    "assert",
                    '!',
                    parens((
                        (
                            output_eq,
                            S,
                            "output_eq",
                            parens(('&', "value", ',', '&', "expected")),
                        ),
                        ',',
                        message,
                        ',',
                        ("value", ',', "expected"),
                    )),
                    ';',
                ),
            );
        }
    }
}

enum CompareExpect<'a> {
    Ignore,
    Expected(&'a TokenTree),
}

impl IntoTokens for CompareExpect<'_> {
    fn into_tokens(self, stream: &mut TokenStream, span: Span) {
        match self {
            CompareExpect::Ignore => {
                stream.write(span, "None");
            }
            CompareExpect::Expected(value) => {
                stream.write(span, "Some");
                stream.write(span, parens(value.clone()));
            }
        }
    }
}
