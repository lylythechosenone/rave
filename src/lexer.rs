pub mod tokens;

use crate::error::Result;
use core::{
    any::{Any, TypeId},
    mem::MaybeUninit,
    ops::Range,
};

pub trait Token: AsAny {
    // dyn-able
    fn span(&self) -> Range<usize>;
    // not dyn-able
    fn parse(start: usize, input: &str) -> Option<Result<(Self, usize)>>
    where
        Self: Sized;
}
pub trait AsAny {
    fn as_any(&self) -> &dyn Any;
    fn type_id() -> TypeId
    where
        Self: Sized;
}
impl<T: Any + Token> AsAny for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn type_id() -> TypeId
    where
        Self: Sized,
    {
        TypeId::of::<Self>()
    }
}

#[repr(align(16))]
#[derive(Debug)]
pub struct Aligned16Bytes([MaybeUninit<u8>; 16]);

#[derive(Debug)]
pub struct TokenBox {
    data: Aligned16Bytes,
    type_id: TypeId,
}
impl TokenBox {
    pub unsafe fn downcast<T>(self) -> T {
        let mut result = MaybeUninit::uninit();
        let src = &self.data as *const _ as *const u8;
        let dst = &mut result as *mut _ as *mut u8;
        unsafe {
            core::ptr::copy_nonoverlapping(src, dst, core::mem::size_of::<T>());
        }
        result.assume_init()
    }
    pub unsafe fn downcast_ref<T>(&self) -> &T {
        unsafe { &*(&self.data as *const _ as *const T) }
    }
    pub unsafe fn downcast_mut<T>(&mut self) -> &mut T {
        unsafe { &mut *(&mut self.data as *mut _ as *mut T) }
    }
    pub fn is<T: 'static>(&self) -> bool {
        self.type_id == TypeId::of::<T>()
    }
    /// ## Panics
    /// Panics if size or align of `T` > 16
    pub fn new<T: 'static>(value: T) -> Self {
        assert!(core::mem::size_of::<T>() <= 16);
        assert!(core::mem::align_of::<T>() <= 16);
        let mut array = Aligned16Bytes([MaybeUninit::uninit(); 16]);
        let src = &value as *const _ as *const u8;
        let dst = &mut array as *mut _ as *mut u8;
        unsafe {
            core::ptr::copy_nonoverlapping(src, dst, core::mem::size_of::<T>());
        }
        Self {
            data: array,
            type_id: TypeId::of::<T>(),
        }
    }
}

pub struct Lexer<'a, const LOOKAHEAD: usize> {
    input: &'a str,
    index: usize,
    buf: heapless::Deque<TokenBox, LOOKAHEAD>,
}
impl<'a, const LOOKAHEAD: usize> Lexer<'a, LOOKAHEAD> {
    /// ## Panics
    /// Panics if size or align of `T` > 16
    pub fn peek<T: Token + 'static>(&mut self) -> Option<Result<&T>> {
        if !self.buf.is_empty() {
            if self.buf.back().unwrap().is::<T>() {
                let val = self.buf.back().unwrap();
                let downcasted = unsafe { val.downcast_ref::<T>() };
                return Some(Ok(downcasted));
            }
        }
        let token = match T::parse(self.index, self.input) {
            Some(Ok(token)) => {
                self.index = token.1;
                self.input = &self.input[self.index..];
                self.trim();
                TokenBox::new(token.0)
            }
            Some(Err(err)) => return Some(Err(err)),
            None => return None,
        };
        self.buf.push_back(token).expect("Out of space");
        Some(Ok(unsafe { self.buf.back().unwrap().downcast_ref() }))
    }
    /// ## Panics
    /// Panics if:
    /// - `n - 1` has not been previously peeked.
    /// - size or align of `T` > 16
    pub fn peek_n<T: Token + 'static>(&mut self, n: usize) -> Option<Result<&T>> {
        if self.buf.len() > n {
            if self.buf.iter().nth(n).unwrap().is::<T>() {
                let val = self.buf.iter().nth(n).unwrap();
                let downcasted = unsafe { val.downcast_ref::<T>() };
                return Some(Ok(downcasted));
            }
        }
        assert_eq!(self.buf.len(), n);
        let token = match T::parse(self.index, self.input) {
            Some(Ok(token)) => {
                self.index = token.1;
                self.input = &self.input[self.index..];
                self.trim();
                TokenBox::new(token.0)
            }
            Some(Err(err)) => return Some(Err(err)),
            None => return None,
        };
        self.buf.push_back(token).expect("Out of space");
        Some(Ok(unsafe { self.buf.back().unwrap().downcast_ref() }))
    }
    pub fn get<T: Token + 'static>(&mut self) -> Option<Result<T>> {
        if !self.buf.is_empty() {
            if self.buf.iter().last().unwrap().is::<T>() {
                let val = self.buf.pop_front().unwrap();
                let downcasted = unsafe { val.downcast::<T>() };
                return Some(Ok(downcasted));
            }
        }
        T::parse(self.index, self.input).map(|res| {
            res.map(|val| {
                self.index = val.1;
                self.input = &self.input[self.index..];
                self.trim();
                val.0
            })
        })
    }
    fn trim(&mut self) {
        while self.input.starts_with(char::is_whitespace) {
            self.index += 1;
            self.input = &self.input[self.index..];
        }
    }
}
