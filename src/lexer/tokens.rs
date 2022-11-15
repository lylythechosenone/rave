use crate::lexer::Lexer;
use crate::{error::Result, lexer::Token};
use core::ops::Range;
use core::str::FromStr;

macro_rules! simple_token {
    ($token:expr => $name:ident) => {
        pub struct $name(::core::ops::Range<usize>);
        impl $crate::lexer::Token for $name {
            fn span(&self) -> &::core::ops::Range<usize> {
                &self.0
            }
            fn parse(start: usize, input: &str) -> Option<$crate::error::Result<(Self, usize)>>
            where
                Self: Sized,
            {
                input.strip_prefix($token).map(|new| {
                    let consumed = input.len() - new.len();
                    Ok((Self(start..start + consumed), consumed))
                })
            }
        }
    };
}

simple_token!('+' => Plus);
simple_token!('-' => Minus);
simple_token!('*' => Star);
simple_token!('/' => Slash);
simple_token!('%' => Percent);
simple_token!("+=" => PlusEqual);
simple_token!("-=" => MinusEqual);
simple_token!("*=" => StarEqual);
simple_token!("/=" => SlashEqual);
simple_token!("%=" => PercentEqual);

simple_token!("==" => EqualEqual);
simple_token!("!=" => BangEqual);
simple_token!("<=" => LessEqual);
simple_token!(">=" => GreaterEqual);

simple_token!("&&" => AndAnd);
simple_token!("||" => OrOr);

simple_token!("!" => Bang);
simple_token!(":=" => ColonEqual);
simple_token!("<" => Less);
simple_token!(">" => Greater);

simple_token!(";" => Semicolon);
simple_token!("," => Comma);
simple_token!("." => Dot);
simple_token!(':' => Colon);

simple_token!("(" => LeftParen);
simple_token!(")" => RightParen);
simple_token!("{" => LeftBrace);
simple_token!("}" => RightBrace);
simple_token!("[" => LeftBracket);
simple_token!("]" => RightBracket);

pub struct Ident(Range<usize>);
impl Ident {
    pub fn eval<'a, const LOOKAHEAD: usize>(&self, lexer: &'a Lexer<'a, LOOKAHEAD>) -> &'a str {
        &lexer.input[self.0.clone()]
    }
}
impl Token for Ident {
    fn span(&self) -> &Range<usize> {
        &self.0
    }
    fn parse(start: usize, input: &str) -> Option<Result<(Self, usize)>>
    where
        Self: Sized,
    {
        input.strip_prefix(char::is_alphabetic).map(|new: &str| {
            let consumed = input.len() - new.len()
                + new.chars().take_while(|c: &char| c.is_alphabetic()).count();
            Ok((Self(start..start + consumed), consumed))
        })
    }
}

pub struct Number(Range<usize>);
impl Number {
    pub fn eval<'a, const LOOKAHEAD: usize, T>(&self, lexer: &'a Lexer<'a, LOOKAHEAD>) -> T
    where
        T: FromStr,
        T::Err: core::fmt::Debug,
    {
        lexer.input[self.0.clone()].parse().unwrap()
    }
}
impl Token for Number {
    fn span(&self) -> &Range<usize> {
        &self.0
    }
    fn parse(start: usize, input: &str) -> Option<Result<(Self, usize)>>
    where
        Self: Sized,
    {
        let mut chars = input.chars();
        match chars.next() {
            Some('0') if chars.next() == Some('x') => {
                let consumed = input.len() - chars.as_str().len()
                    + chars.take_while(|c: &char| c.is_ascii_hexdigit()).count();
                Some(Ok((Self(start..start + consumed), consumed)))
            }
            Some('0') if chars.next() == Some('b') => {
                let consumed = input.len() - chars.as_str().len()
                    + chars.take_while(|c: &char| c == &'0' || c == &'1').count();
                Some(Ok((Self(start..start + consumed), consumed)))
            }
            Some(c) if c.is_ascii_digit() => {
                let consumed = input.len() - chars.as_str().len()
                    + chars
                        .take_while(|c: &char| c.is_ascii_digit() || c == &'.')
                        .count();
                Some(Ok((Self(start..start + consumed), consumed)))
            }
            _ => None,
        }
    }
}
