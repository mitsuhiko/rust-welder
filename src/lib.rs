#![crate_name = "welder"]
#![crate_type = "lib"]
#![license = "BSD"]
#![comment = "Rust error experimentation."]

#![deny(non_camel_case_types)]
#![feature(macro_rules)]
#![feature(associated_types)]

use std::{raw, mem, fmt};
use std::intrinsics::TypeId;


/// Holds information related to the location of an error.
#[deriving(Eq, PartialEq, Clone)]
pub struct ErrorLocation {
    /// the file that caused the error.
    pub file: Path,
    /// the line in the file that caused the error.
    pub line: uint,
    /// the column in the file that caused the error.
    pub col: uint,
}

impl fmt::Show for ErrorLocation {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}:{}:{}",
            self.file.display(),
            self.line,
            self.col,
        )
    }
}


/// Trait that represents errors.
pub trait Error: 'static + Send {

    /// The name of the error.
    fn name(&self) -> &str;

    /// The description of the error.
    fn description(&self) -> &str;

    /// A detailed description of the error, usually including
    /// dynamic information.
    fn detail(&self) -> Option<String> { None }

    /// The lower-level cause of this error, if any.
    fn cause(&self) -> Option<&Error> { None }

    /// The location of this error if available.
    fn location(&self) -> Option<ErrorLocation> { None }

    /// This apparently needs to be here instead of 
    #[doc(hidden)]
    fn get_error_type(&self) -> TypeId { TypeId::of::<Self>() }
}

pub trait ErrorExt<'a> {
    fn cast<E: Error>(self) -> Option<&'a E>;
}

impl<'a> ErrorExt<'a> for &'a Error {

    #[inline(always)]
    fn cast<E: Error>(self) -> Option<&'a E> {
        if self.get_error_type() == TypeId::of::<E>() {
            unsafe {
                let to: raw::TraitObject = mem::transmute_copy(&self);
                Some(mem::transmute(to.data))
            }
        } else {
            None
        }
    }
}


/// A common error data struct.  A boxed version of this is should be used
/// by fat errors for efficiency reasons.
#[deriving(Clone)]
pub struct CommonErrorData<K: Eq> {
    pub kind: K,
    pub description: &'static str,
    pub detail: Option<String>,
    pub location: Option<ErrorLocation>,
}


/// Helper trait that is used with the fail! macro for error creation.
/// The arguments of the fail! macro are provided as tuple in the first
/// argument to `construct_error`, the error location is provided as
/// optional second argument.
pub trait ConstructError<A> {
    fn construct_error(args: A, loc: Option<ErrorLocation>) -> Self;
}

/// This trait is used by `try!` and `propagate_error!` to convert and
/// propagate errors.
pub trait FromError<E> {
    fn from_error(err: E, loc: Option<ErrorLocation>) -> Self;
}

/// Each type can transparently convert to itself.  The error location
/// is ignored in that case.
impl<E> FromError<E> for E {
    fn from_error(err: E, _: Option<ErrorLocation>) -> E {
        err
    }
}

/// Provides the error location for the place where the macro expands.
/// It's a better idea to use `debug_error_location!` instead which
/// will expand to an optional error location.
#[macro_export]
macro_rules! error_location {
    () => ({
        ::welder::ErrorLocation {
            file: ::std::path::Path::new(file!()),
            line: line!(),
            col: col!(),
        }
    })
}

/// Like `error_location!` but expands into an option of the error location
/// which is only filled in for debug builds.
#[macro_export]
macro_rules! debug_error_location {
    () => ({
        if cfg!(ndebug) {
            None
        } else {
            Some(error_location!())
        }
    })
}

/// Constructs an error and aborts with it.
#[macro_export]
macro_rules! fail {
    ($($expr:expr),*) => ({
        return Err(::welder::ConstructError::construct_error(
            ($($expr,)*), debug_error_location!()))
    });
}

/// Propagates an error forward.  This will perform automatic error conversion
/// for the provided error.
#[macro_export]
macro_rules! propagate_error {
    ($expr:expr) => ({
        return Err(::welder::FromError::from_error($expr, debug_error_location!()))
    });
}

/// Tries to unwrap a result and if that fails, will propagate the error
/// forward with `propagate_error!`.
#[macro_export]
macro_rules! try {
    ($expr:expr) => (match $expr {
        Err(x) => propagate_error!(x),
        Ok(x) => x,
    })
}
