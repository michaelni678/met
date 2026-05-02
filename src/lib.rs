//! **Met** provides several macros and extension traits that make working with
//! [`proc-macro2`] easier.
//!
//! # Setup
//!
//! Add this to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! met = "0.1.0"
//! proc-macro2 = "1"
//! ```
//! 
//! [`proc-macro2`]: https://crates.io/crates/proc-macro2

use proc_macro2::{Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenStream, TokenTree};

mod sealed {
    pub trait Sealed {}
}

/// [`TokenStream`] quasi-quoting macro.
///
/// This is not a replacement for [`quote::quote!`]. It's slower and does not
/// support variable interpolation. It is intended for cases where
/// interpolation must be avoided entirely.
///
/// # Examples
///
/// ```
/// # use met::assert_stream_eq;
/// use met::stream;
/// use quote::quote;
///
/// let a = stream! {
///     impl MyTrait for MyStruct {}
/// };
///
/// let b = quote! {
///     impl MyTrait for MyStruct {}
/// };
///
/// assert_stream_eq!(a, b);
/// ```
///
/// When `#` is followed by an ident, `quote!` performs variable
/// interpolation, but `stream!` does not.
///
/// ```
/// # use met::assert_stream_ne;
/// use met::stream;
/// use quote::quote;
///
/// let variable = 5;
///
/// let a = stream! { # variable };
/// let b = quote! { # variable };
///
/// // Not equal, `stream!` doesn't interpolate!
/// assert_stream_ne!(a, b);
/// ```
///
/// [`quote::quote!`]: https://docs.rs/quote/latest/quote/macro.quote.html
#[macro_export]
macro_rules! stream {
    ($($tt:tt)*) => {
        stringify!($($tt)*)
            .parse::<::proc_macro2::TokenStream>()
            .expect("couldn't parse tokens")
    };
}

/// [`TokenTree`] quasi-quoting macro.
#[macro_export]
macro_rules! tree {
    ($tt:tt) => {
        $crate::stream!($tt)
            .into_iter()
            .next()
            .expect("expected a token")
    };
}

/// [`Group`] quasi-quoting macro.
///
/// # Examples
///
/// ```
/// # use met::{GroupExt, assert_stream_eq, stream};
/// use met::group;
///
/// let group = group! { [1, 2, 3] };
///
/// assert!(group.is_bracketed());
/// assert_stream_eq!(group.stream(), stream! { 1, 2, 3 });
/// ```
///
/// If the token isn't a group, the macro will panic.
///
/// ```should_panic
/// use met::group;
///
/// group! { "met" };
/// ```
#[macro_export]
macro_rules! group {
    ($tt:tt) => {
        $crate::TokenTreeExt::into_group($crate::tree!($tt)).expect("expected a group")
    };
}

/// [`Ident`] quasi-quoting macro.
///
/// # Examples
///
/// ```
/// use met::ident;
///
/// let ident = ident! { variable };
///
/// assert_eq!(ident, "variable");
/// ```
///
/// If the token isn't an ident, the macro will panic.
///
/// ```should_panic
/// use met::ident;
///
/// ident! { [1, 2, 3] };
/// ```
#[macro_export]
macro_rules! ident {
    ($tt:tt) => {
        $crate::TokenTreeExt::into_ident($crate::tree!($tt)).expect("expected an ident")
    };
}

/// [`Punct`] quasi-quoting macro.
///
/// # Examples
///
/// ```
/// # use met::PunctExt;
/// use met::punct;
///
/// let punct = punct! { < };
///
/// assert!(punct.is_char('<'));
/// assert!(punct.is_alone());
/// ```
///
/// If the token isn't a punct, the macro will panic.
///
/// ```should_panic
/// use met::punct;
///
/// punct! { variable };
/// ```
#[macro_export]
macro_rules! punct {
    ($tt:tt) => {
        $crate::TokenTreeExt::into_punct($crate::tree!($tt)).expect("expected a punct")
    };
}

/// [`Literal`] quasi-quoting macro.
///
/// # Examples
///
/// ```
/// use met::literal;
///
/// let literal = literal! { "met" };
///
/// assert_eq!(literal.to_string(), "\"met\"");
/// ```
///
/// If the token isn't a literal, the macro will panic.
///
/// ```should_panic
/// use met::literal;
///
/// literal! { < };
/// ```
#[macro_export]
macro_rules! literal {
    ($tt:tt) => {
        $crate::TokenTreeExt::into_literal($crate::tree!($tt)).expect("expected a literal")
    };
}

/// Convenience macro for checking token stream equality.
#[macro_export]
macro_rules! assert_stream_eq {
    ($left:expr, $right:expr) => {
        assert!($crate::TokenStreamExt::equals(&$left, &$right));
    };
}

/// Convenience macro for checking token stream inequality.
#[macro_export]
macro_rules! assert_stream_ne {
    ($left:expr, $right:expr) => {
        assert!(!$crate::TokenStreamExt::equals(&$left, &$right));
    };
}

/// Extension trait for [`TokenStream`].
pub trait TokenStreamExt: Sized + sealed::Sealed {
    /// Checks if the token stream is equal to `other`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use met::stream;
    /// use met::TokenStreamExt;
    ///
    /// let a = stream! {
    ///     if true {
    ///         return 8;
    ///     }
    /// };
    ///
    /// let b = stream! {
    ///     if
    ///         true { return
    ///         8; }
    /// };
    ///
    /// // Equal, the tokens are identical!
    /// assert!(a.equals(&b));
    ///
    /// let c = stream! {
    ///     if true {
    ///         return 8
    ///     }
    /// };
    ///
    /// // Not equal, `c` is missing a semicolon (;) after `8`.
    /// assert!(!a.equals(&c));
    /// ```
    fn equals(&self, other: &Self) -> bool;

    /// Append a token to the token stream.
    ///
    /// # Examples
    ///
    /// ```
    /// # use met::{assert_stream_eq, stream};
    /// # use proc_macro2::{Ident, Literal, Punct, Spacing, Span, TokenStream};
    /// use met::TokenStreamExt;
    ///
    /// let mut expression = TokenStream::new();
    ///
    /// expression.append(Ident::new("variable", Span::call_site()));
    /// expression.append(Punct::new('=', Spacing::Joint));
    /// expression.append(Punct::new('=', Spacing::Alone));
    /// expression.append(Literal::isize_unsuffixed(100));
    ///
    /// assert_stream_eq!(expression, stream! { variable == 100 });
    /// ```
    fn append<T>(&mut self, token: T)
    where
        T: Into<TokenTree>;
}

impl TokenStreamExt for TokenStream {
    fn equals(&self, other: &Self) -> bool {
        let mut this = self.clone().into_iter();
        let mut other = other.clone().into_iter();

        loop {
            match (this.next(), other.next()) {
                (Some(this), Some(other)) if this.equals(&other) => {}
                (None, None) => return true,
                _ => return false,
            }
        }
    }

    fn append<T>(&mut self, token: T)
    where
        T: Into<TokenTree>,
    {
        self.extend(Some(token.into()));
    }
}

impl sealed::Sealed for TokenStream {}

/// Extension trait for [`TokenTree`].
pub trait TokenTreeExt: Sized + sealed::Sealed {
    /// Checks if this token tree is equal to `other`.
    fn equals(&self, other: &Self) -> bool;

    /// Checks if the token tree is a group.
    fn is_group(&self) -> bool;

    /// Checks if the token tree is an ident.
    fn is_ident(&self) -> bool;

    /// Checks if the token tree is a punct.
    fn is_punct(&self) -> bool;

    /// Checks if the token tree is a literal.
    fn is_literal(&self) -> bool;

    /// Require the token tree to be a group.
    fn into_group(self) -> Result<Group, Self>;

    /// Require the token tree to be an ident.
    fn into_ident(self) -> Result<Ident, Self>;

    /// Require the token tree to be a punct.
    fn into_punct(self) -> Result<Punct, Self>;

    /// Require the token tree to be a literal.
    fn into_literal(self) -> Result<Literal, Self>;

    /// Returns the token tree as a group if it is one.
    fn as_group(&self) -> Option<&Group>;

    /// Returns the token tree as an ident if it is one.
    fn as_ident(&self) -> Option<&Ident>;

    /// Returns the token tree as a punct if it is one.
    fn as_punct(&self) -> Option<&Punct>;

    /// Returns the token tree as a literal if it is one.
    fn as_literal(&self) -> Option<&Literal>;
}

impl TokenTreeExt for TokenTree {
    fn is_group(&self) -> bool {
        matches!(self, Self::Group(_))
    }

    fn is_ident(&self) -> bool {
        matches!(self, Self::Ident(_))
    }

    fn is_punct(&self) -> bool {
        matches!(self, Self::Punct(_))
    }

    fn is_literal(&self) -> bool {
        matches!(self, Self::Literal(_))
    }

    fn equals(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Group(this), Self::Group(other)) => this.equals(other),
            (Self::Ident(this), Self::Ident(other)) => this.equals(other),
            (Self::Punct(this), Self::Punct(other)) => this.equals(other),
            (Self::Literal(this), Self::Literal(other)) => this.equals(other),
            _ => false,
        }
    }

    fn into_group(self) -> Result<Group, Self> {
        match self {
            Self::Group(group) => Ok(group),
            _ => Err(self),
        }
    }

    fn into_ident(self) -> Result<Ident, Self> {
        match self {
            Self::Ident(ident) => Ok(ident),
            _ => Err(self),
        }
    }

    fn into_punct(self) -> Result<Punct, Self> {
        match self {
            Self::Punct(punct) => Ok(punct),
            _ => Err(self),
        }
    }

    fn into_literal(self) -> Result<Literal, Self> {
        match self {
            Self::Literal(literal) => Ok(literal),
            _ => Err(self),
        }
    }

    fn as_group(&self) -> Option<&Group> {
        match self {
            Self::Group(group) => Some(group),
            _ => None,
        }
    }

    fn as_ident(&self) -> Option<&Ident> {
        match self {
            Self::Ident(ident) => Some(ident),
            _ => None,
        }
    }

    fn as_punct(&self) -> Option<&Punct> {
        match self {
            Self::Punct(punct) => Some(punct),
            _ => None,
        }
    }

    fn as_literal(&self) -> Option<&Literal> {
        match self {
            Self::Literal(literal) => Some(literal),
            _ => None,
        }
    }
}

impl sealed::Sealed for TokenTree {}

/// Extension trait for [`Group`].
pub trait GroupExt: Sized + sealed::Sealed {
    /// Construct a group with the given span, delimiter, and token stream.
    fn new_spanned(span: Span, delimiter: Delimiter, stream: TokenStream) -> Self;

    /// Checks if this group is equal to `other`.
    fn equals(&self, other: &Self) -> bool;

    /// Checks if the delimiter is [`Delimiter::Parenthesis`].
    fn is_parenthesized(&self) -> bool;

    /// Checks if the delimiter is [`Delimiter::Brace`].
    fn is_braced(&self) -> bool;

    /// Checks if the delimiter is [`Delimiter::Bracket`].
    fn is_bracketed(&self) -> bool;
}

impl GroupExt for Group {
    fn new_spanned(span: Span, delimiter: Delimiter, stream: TokenStream) -> Self {
        let mut group = Group::new(delimiter, stream);
        group.set_span(span);
        group
    }

    fn equals(&self, other: &Self) -> bool {
        self.delimiter() == other.delimiter() && self.stream().equals(&other.stream())
    }

    fn is_parenthesized(&self) -> bool {
        self.delimiter() == Delimiter::Parenthesis
    }

    fn is_braced(&self) -> bool {
        self.delimiter() == Delimiter::Brace
    }

    fn is_bracketed(&self) -> bool {
        self.delimiter() == Delimiter::Bracket
    }
}

impl sealed::Sealed for Group {}

/// Extension trait for [`Ident`].
pub trait IdentExt: Sized + sealed::Sealed {
    /// Checks if this ident is equal to `other`.
    ///
    /// This method is provided for completeness and simply delegates to the
    /// [`PartialEq`] implementation.
    fn equals(&self, other: &Self) -> bool;
}

impl IdentExt for Ident {
    fn equals(&self, other: &Self) -> bool {
        self == other
    }
}

impl sealed::Sealed for Ident {}

/// Extension trait for [`Punct`].
pub trait PunctExt: Sized + sealed::Sealed {
    /// Checks if this punct is equal to `other`.
    fn equals(&self, other: &Self) -> bool;

    /// Checks if the punct character is `character`.
    fn is_char(&self, character: char) -> bool;

    /// Checks if the spacing is [`Spacing::Alone`].
    fn is_alone(&self) -> bool;

    /// Checks if the spacing is [`Spacing::Joint`].
    fn is_joint(&self) -> bool;
}

impl PunctExt for Punct {
    fn equals(&self, other: &Self) -> bool {
        self.as_char() == other.as_char() && self.spacing() == other.spacing()
    }

    fn is_char(&self, character: char) -> bool {
        self.as_char() == character
    }

    fn is_alone(&self) -> bool {
        self.spacing() == Spacing::Alone
    }

    fn is_joint(&self) -> bool {
        self.spacing() == Spacing::Joint
    }
}

impl sealed::Sealed for Punct {}

/// Extension trait for [`Literal`].
pub trait LiteralExt: Sized + sealed::Sealed {
    /// Checks if this literal is equal to `other`.
    fn equals(&self, other: &Self) -> bool;
}

impl LiteralExt for Literal {
    fn equals(&self, other: &Self) -> bool {
        self.to_string() == other.to_string()
    }
}

impl sealed::Sealed for Literal {}
