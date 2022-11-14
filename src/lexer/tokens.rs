macro_rules! simple_token {
    ($token:expr => $name:ident) => {
        pub struct $name(::core::ops::Range<usize>);
        impl $crate::lexer::Token for $name {
            fn span(&self) -> ::core::ops::Range<usize> {
                self.0.clone()
            }
            fn parse(start: usize, input: &str) -> Option<$crate::error::Result<(Self, usize)>>
            where
                Self: Sized,
            {
                input.strip_prefix($token).map(|new| {
                    let end = start + (input.len() - new.len());
                    Ok((Self(start..end), end))
                })
            }
        }
    };
}

simple_token!('+' => Plus);
simple_token!('-' => Minus);
simple_token!('*' => Star);
simple_token!('/' => Slash);
