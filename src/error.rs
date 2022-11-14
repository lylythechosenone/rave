#[derive(Debug)]
pub enum Error<'a> {
    UnexpectedToken {
        unexpected: &'a str,
        expected: &'a str,
    },
}

pub type Result<'a, T> = core::result::Result<T, Error<'a>>;
