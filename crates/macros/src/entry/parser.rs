use proc_macro::{Delimiter, Literal, Spacing, Span, TokenTree};

use crate::entry::output::{Config, ItemOutput};
use crate::error::Error;
use crate::parsing::{BaseParser, Buf};
use crate::parsing::{Punct, COMMA, EQ};

/// A parser for the arguments provided to an entry macro.
pub(crate) struct ConfigParser<'a> {
    base: BaseParser<'a>,
    errors: &'a mut Vec<Error>,
}

impl<'a> ConfigParser<'a> {
    /// Construct a new parser around the given token stream.
    pub(crate) fn new(
        stream: proc_macro::TokenStream,
        buf: &'a mut Buf,
        errors: &'a mut Vec<Error>,
    ) -> Self {
        Self {
            base: BaseParser::new(stream, buf),
            errors,
        }
    }

    /// Parse and produce the corresponding token stream.
    pub(crate) fn parse(mut self) -> Config {
        let mut config = Config::default();

        while self.base.nth(0).is_some() {
            if self.parse_option(&mut config).is_none() {
                self.recover();
                continue;
            }

            if !self.base.skip_punct(COMMA) {
                break;
            }
        }

        if let Some(tt) = self.base.nth(0) {
            self.errors.push(Error::new(tt.span(), "trailing token"));
        }

        config
    }

    /// Recover by parsing either to the next comma `,`, or end of input.
    fn recover(&mut self) {
        loop {
            if let Some(p @ Punct { chars: COMMA, .. }) = self.base.peek_punct() {
                self.base.step(p.len());
                break;
            }

            if self.base.bump().is_none() {
                break;
            }
        }
    }

    /// Parse a single option.
    fn parse_option(&mut self, config: &mut Config) -> Option<()> {
        match self.base.bump() {
            Some(TokenTree::Ident(ident)) => match self.base.buf.display_as_str(&ident) {
                "input" => {
                    self.parse_eq()?;
                    config.input_file = Some(self.parse_literal()?);
                    Some(())
                }
                name => {
                    self.errors.push(Error::new(
                        ident.span(),
                        format!("unknown option `{}`", name),
                    ));
                    None
                }
            },
            tt => {
                let span = tt.map(|tt| tt.span()).unwrap_or_else(Span::call_site);
                self.errors.push(Error::new(span, "expected identifier"));
                None
            }
        }
    }

    /// Parse the next element as a literal value.
    fn parse_literal(&mut self) -> Option<Literal> {
        match self.base.bump() {
            Some(TokenTree::Literal(literal)) => Some(literal),
            tt => {
                let span = tt.map(|tt| tt.span()).unwrap_or_else(Span::call_site);
                self.errors.push(Error::new(span, "expected literal"));
                None
            }
        }
    }

    /// Parse the next element as an `=` punctuation.
    fn parse_eq(&mut self) -> Option<()> {
        match self.base.peek_punct()? {
            p @ Punct { chars: EQ, .. } => {
                self.base.step(p.len());
                Some(())
            }
            p => {
                self.errors
                    .push(Error::new(p.span, "expected assignment `=`"));
                None
            }
        }
    }
}

/// A parser for the item annotated with an entry macro.
pub(crate) struct ItemParser<'a> {
    base: BaseParser<'a>,
}

impl<'a> ItemParser<'a> {
    /// Construct a new parser around the given token stream.
    pub(crate) fn new(stream: proc_macro::TokenStream, buf: &'a mut Buf) -> Self {
        Self {
            base: BaseParser::new(stream, buf),
        }
    }

    /// Parse and produce the corresponding token stream.
    pub(crate) fn parse(mut self) -> ItemOutput {
        let start = self.base.len();
        let mut signature = None;
        let mut block = None;
        let mut fn_name = None;
        let mut next_is_name = false;
        let mut args = None;

        while let Some(tt) = self.base.bump() {
            match &tt {
                TokenTree::Ident(ident) => match self.base.buf.display_as_str(&ident) {
                    "fn" => {
                        if fn_name.is_none() {
                            next_is_name = true;
                        }
                    }
                    _ => {
                        if std::mem::take(&mut next_is_name) {
                            fn_name = Some(self.base.len());
                        }
                    }
                },
                TokenTree::Group(g) => match g.delimiter() {
                    Delimiter::Parenthesis if args.is_none() && fn_name.is_some() => {
                        args = Some(self.base.len() - start);
                    }
                    Delimiter::Brace if block.is_none() => {
                        signature = Some(start..self.base.len());
                        block = Some(self.base.len());
                    }
                    _ => {}
                },
                TokenTree::Punct(p) if p.as_char() == '<' && p.spacing() == Spacing::Alone => {
                    self.base.push(tt);
                    self.skip_angle_brackets();
                    continue;
                }
                _ => {}
            }

            self.base.push(tt);
        }

        let tokens = self.base.into_tokens();

        ItemOutput::new(tokens, fn_name, signature, block, args)
    }

    /// Since generics are implemented using angle brackets.
    fn skip_angle_brackets(&mut self) {
        // NB: one bracket encountered already.
        let mut level = 1u32;

        while let Some(tt) = self.base.bump() {
            match &tt {
                TokenTree::Punct(p) if p.as_char() == '<' && p.spacing() == Spacing::Alone => {
                    level += 1;
                }
                TokenTree::Punct(p) if p.as_char() == '>' && p.spacing() == Spacing::Alone => {
                    level -= 1;
                }
                _ => {}
            }

            self.base.push(tt);

            if level == 0 {
                break;
            }
        }
    }
}
