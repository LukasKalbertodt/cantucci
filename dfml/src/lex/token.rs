//! This module defines all token types.
//!

use std::fmt::{Display, Formatter, Error};
use std::str::FromStr;
use base::Span;

/// A token with its span in the source code.
#[derive(Debug, Clone, PartialEq)]
pub struct TokenSpan<'src> {
    /// The token
    pub tok: Token<'src>,

    /// Byte position of token in Filemap
    pub span: Span,
}

// Macro to generate enums with helper methods
macro_rules! gen_helper {
    (
        $name:ident; ;
        $($variant:ident = $val:expr),+
    ) => { };
    (
        $name:ident;
        $helper:ident $(, $tail:ident)*;
        $($variant:ident = $val:expr),+
    ) => {
        $helper!($name; $($variant = $val),+ );
        gen_helper!($name; $($tail),*; $($variant = $val),+);
    };
}

macro_rules! gen_enum {
    (
        $(#[$attr:meta])*
        pub enum $name:ident;
        with $($helper:ident),* for:
        $($variant:ident = $val:expr),+
    ) => {
        $(
            #[$attr]
        )*
        pub enum $name {
            $($variant,)*
        }
        gen_helper!($name; $($helper),*; $( $variant = $val ),+);
    }
}

macro_rules! as_str {
    ($name:ident; $($variant:ident = $val:expr),+) => {
        impl $name {
            pub fn as_str(&self) -> &'static str {
                match self {
                    $( &$name::$variant => $val ,)*
                }
            }
        }
    }
}

macro_rules! display {
    ($name:ident; $($variant:ident = $val:expr),+) => {
        impl Display for $name {
            fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
                self.as_str().fmt(f)
            }
        }
    }
}

macro_rules! from_str {
    ($name:ident; $($variant:ident = $val:expr),+) => {
        impl FromStr for $name {
            type Err = ();
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    $($val => Ok($name::$variant), )*
                    _ => Err(()),
                }
            }
        }
    }
}

/// A token.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token<'src> {
    Ident(&'src str),
    Keyword(Keyword),

    // Some kind of separators
    // (   )   {   }   [   ]   ;   ,   .   :   ::
    ParenOp,
    ParenCl,
    BraceOp,
    BraceCl,
    BracketOp,
    BracketCl,
    Semi,
    Comma,
    Dot,
    Colon,
    ColonColon,
}

impl<'src> Token<'src> {
    pub fn as_str(&self) -> &'src str {
        use self::Token::*;

        match *self {
            Ident(ident) => ident,
            Keyword(kw) => kw.as_str(),
            ParenOp => "(",
            ParenCl => ")",
            BraceOp => "{",
            BraceCl => "}",
            BracketOp => "[",
            BracketCl => "]",
            Semi => ";",
            Comma => ",",
            Dot => ".",
            Colon => ":",
            ColonColon => "::",
        }
    }
}


gen_enum! {
    /// Represents a keyword
    #[derive(Copy, Clone, PartialEq, Eq, Debug)]
    pub enum Keyword;
    with as_str, display, from_str for:

    Shape = "shape",
    Param = "param"
}
