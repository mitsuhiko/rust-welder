//! Welder is a crate that acts as an experiment for error interoperabilty
//! in Rust.  The goal is to provide good error reporting and handling
//! functionality.
#![crate_name = "welder"]
#![crate_type = "lib"]
#![license = "BSD"]
#![comment = "Rust error experimentation."]

#![deny(non_camel_case_types)]
#![feature(macro_rules)]
#![feature(associated_types)]
#![feature(while_let)]

use std::{raw, mem, fmt};
use std::intrinsics::TypeId;
use std::io;


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

impl ErrorLocation {

    /// Returns the source line for this error location if this is possible.
    fn get_source_line(&self) -> io::IoResult<String> {
        let file = try!(io::File::open(&self.file));
        let mut reader = io::BufferedReader::new(file);
        match reader.lines().skip(self.line - 1).next() {
            Some(Ok(line)) => Ok(line),
            _ => Err(io::IoError {
                kind: io::EndOfFile,
                desc: "Reached end of file unexpectedly",
                detail: None,
            }),
        }
    }
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
///
/// Example:
///
/// ```rust
/// impl Error for MyError {
///     fn name(&self) -> &str {
///         "MyError"
///     }
/// 
///     fn description(&self) -> &str {
///         self.data.description
///     }
/// 
///     fn detail(&self) -> Option<String> {
///         self.data.detail.clone()
///     }
/// 
///     fn location(&self) -> Option<ErrorLocation> {
///         self.data.location.clone()
///     }
/// }
/// ```
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

/// Extension methods for errors.
///
/// This provides convenient extra methods for errors.  In the future
/// when the language permits it, this will move to other places.
pub trait ErrorExt<'a> {

    /// Casts an abstract `&Error` into a concrete error.  This only
    /// works if the error is of that actual type which is why this
    /// returns an option.
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

// local hack until $crate lands
mod welder {
    pub use super::{ConstructError, ErrorLocation};
}


/// A common error data struct.
///
/// A boxed version of this is should be used by fat errors for efficiency reasons.
/// This allows the result of errors to stay word sized which helps performance
/// and memory usage.
#[deriving(Clone)]
pub struct CommonErrorData<K: Eq> {
    pub kind: K,
    pub description: &'static str,
    pub detail: Option<String>,
    pub location: Option<ErrorLocation>,
}


/// Helper trait that is used with the `fail!` macro for error creation.
///
/// The arguments of the fail! macro are provided as tuple in the first
/// argument to `construct_error`, the error location is provided as
/// optional second argument.
///
/// Example:
///
/// ``rust
/// impl ConstructError<(MyErrorKind, &'static str)> for MyError {
///     fn construct_error((kind, desc): (MyErrorKind, &'static str),
///                        loc: Option<ErrorLocation>) -> MyError {
///         MyError {
///             data: box CommonErrorData {
///                 description: desc,
///                 kind: kind,
///                 detail: None,
///                 location: loc,
///             }
///         }
///     }
/// }
/// ```
pub trait ConstructError<A> {
    fn construct_error(args: A, loc: Option<ErrorLocation>) -> Self;
}

/// This allows `fail!` with a single argument that is an error to
/// invoke `FromError::from_error` automatically.
impl<S: Error, E: FromError<S>> ConstructError<(S,)> for E {
    #[inline(always)]
    fn construct_error((err,): (S,), loc: Option<ErrorLocation>) -> E {
        FromError::from_error(err, loc)
    }
}

/// This trait is used by `try!` and `fail!` to convert and propagate errors.
///
/// This trait allows to define compatible errors to your own errors so
/// that the `try!` and `fail!` macro will automatically perform wrapping
/// of the error.
///
/// Example:
///
/// ```rust
/// impl FromError<io::IoError> for MyError {
///     fn from_error(err: io::IoError, loc: Option<ErrorLocation>) -> MyError {
///         MyError {
///             data: box CommonErrorData {
///                 description: "an I/O error occurred",
///                 kind: InternalIoError(err),
///                 detail: None,
///                 location: loc,
///             }
///         }
///     }
/// }
/// ```
pub trait FromError<E> {
    fn from_error(err: E, loc: Option<ErrorLocation>) -> Self;
}

/// Each type can transparently convert to itself.  The error location
/// is ignored in that case.
impl<E> FromError<E> for E {
    #[inline(always)]
    fn from_error(err: E, _: Option<ErrorLocation>) -> E {
        err
    }
}

/// Provides the error location for the place where the macro expands.
///
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

/// Like `error_location!` but expands into an option.
///
/// In non debug builds the error information is excluded.
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

/// Aborts with an error.
///
/// This macro fulfills two purposes.  If it's invoked with a single error
/// it will indirectly invoke `FromError::from_error`.  If it's invoked
/// in any other way it will call `ConstructError::construct_error` to
/// construct a new error.  In either case it will include the location
/// information for the current source line in debug builds.
///
/// In either case it will create an early error return with the error
/// created.
///
/// This macro allows propagation of compatible errors as well as failing
/// with new errors.
#[macro_export]
macro_rules! fail {
    ($($expr:expr),*) => ({
        return Err(::welder::ConstructError::construct_error(
            ($($expr,)*), debug_error_location!()))
    });
}

/// Unwraps a value and propagates errors.
///
/// If an expression is wrapped in the `try!` macro this will expand unwrap
/// the success value or propagate the error through `fail!`.
#[macro_export]
macro_rules! try {
    ($expr:expr) => (match $expr {
        Err(x) => fail!(x),
        Ok(x) => x,
    })
}

/// Helper for formatting errors.
pub struct ErrorFormatter<W: Writer> {
    writer: W,
}

/// Helper for formatting errors.
impl<W: Writer> ErrorFormatter<W> {

    /// Creates a new error formatter.
    pub fn new(writer: W) -> ErrorFormatter<W> {
        ErrorFormatter {
            writer: writer,
        }
    }

    /// Formats the entire trace of an error.  This basically invokes the
    /// `format_cause` function for the given error and all causes in
    /// reverse order and adds a header.
    pub fn format_trace(&mut self, err: &Error) -> io::IoResult<()> {
        try!(writeln!(self.writer, "Error causes (most recent error last):"));

        let mut causes = vec![];
        let mut cur_err = Some(err);
        while let Some(x) = cur_err {
            causes.push(x);
            cur_err = x.cause();
        }
        causes.reverse();
        for (idx, cause) in causes.iter().enumerate() {
            if idx != 0 {
                try!(writeln!(self.writer, ""));
            }
            try!(self.format_cause(*cause));
        }

        Ok(())
    }

    /// Formats a s single error cause.
    pub fn format_cause(&mut self, err: &Error) -> io::IoResult<()> {
        match err.location() {
            Some(loc) => {
                try!(writeln!(self.writer, "  File \"{}\", line {}",
                              loc.file.display(), loc.line));
                match loc.get_source_line() {
                    Ok(line) => try!(writeln!(self.writer, "    {}",
                        line.trim_chars([' ', '\t', '\r', '\n'].as_slice()))),
                    Err(_) => {}
                }
            }
            None => {
                try!(writeln!(self.writer, "  File <unknown>, line ?"));
            }
        }

        try!(write!(self.writer, "  {}: {}",
                    err.name(), err.description()));
        match err.detail() {
            Some(detail) => try!(write!(self.writer, " ({})", detail)),
            None => {}
        }
        try!(writeln!(self.writer, ""));
        Ok(())
    }
}

/// Helper function to print the error cause stack to stderr.
pub fn print_error_stack(err: &Error) {
    let mut fmt = ErrorFormatter::new(std::io::stdio::stderr());
    let _ = fmt.format_trace(err);
}


// default implementations of errors
impl Error for io::IoError {

    fn name(&self) -> &str {
        "IoError"
    }

    fn description(&self) -> &str {
        self.desc
    }

    fn detail(&self) -> Option<String> {
        self.detail.clone()
    }
}
