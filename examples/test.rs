#![feature(phase)]

#[phase(plugin, link)]
extern crate welder;

use welder::{Error, ErrorExt, CommonErrorData, ErrorLocation, ConstructError};


fn print_error(err: &Error, cause: &str) {
    println!("{}:", cause);
    println!("  {}: {}", err.name(), err.description());
    match err.detail() {
        Some(detail) => println!("    detail: {}", detail),
        None => {},
    }
    match err.location() {
        Some(location) => println!("    Error location: {}", location),
        None => {},
    }
    match err.cause() {
        Some(cause) => print_error(cause, "Error was caused by"),
        None => {},
    }
}

fn print_error_stack(err: &Error) {
    print_error(err, "An unhandled error occurred");
}

#[deriving(Eq, PartialEq, Clone)]
enum CliErrorKind {
    NotFound,
    NoPermission,
}

#[deriving(Clone)]
struct CliError {
    data: Box<CommonErrorData<CliErrorKind>>,
}

impl Error for CliError {
    fn name(&self) -> &str {
        "CLI Error"
    }

    fn description(&self) -> &str {
        self.data.description
    }

    fn detail(&self) -> Option<String> {
        self.data.detail.clone()
    }

    fn location(&self) -> Option<ErrorLocation> {
        self.data.location.clone()
    }
}

impl ConstructError<(CliErrorKind, &'static str)> for CliError {
    fn construct_error((kind, desc): (CliErrorKind, &'static str),
                       loc: Option<ErrorLocation>) -> CliError {
        CliError {
            data: box CommonErrorData {
                description: desc,
                kind: kind,
                detail: None,
                location: loc,
            }
        }
    }
}

fn test_missing_item() -> Result<(), CliError> {
    fail!(NotFound, "The intended item does not exist.");
}

fn bar() -> Result<(), CliError> {
    try!(test_missing_item());
    fail!(NoPermission, "Access not possible");
}


fn main() {
    match bar() {
        Err(e) => {
            match (&e as &Error).cast::<CliError>() {
                Some(_) => { println!("Got a compatible cli error!"); }
                None => { println!("This was not a cli error!"); }
            }
            print_error_stack(&e);
        },
        Ok(_) => {},
    }
}
