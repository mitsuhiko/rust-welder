#![feature(phase)]

#[phase(plugin, link)]
extern crate welder;

use std::io;

use welder::{Error, ErrorExt, CommonErrorData, ErrorLocation, ConstructError,
             FromError, print_error_stack};


#[deriving(Eq, PartialEq, Clone)]
enum CliErrorKind {
    NotFound,
    NoPermission,
    InternalIoError(io::IoError),
}

#[deriving(Clone)]
struct CliError {
    data: Box<CommonErrorData<CliErrorKind>>,
}

impl Error for CliError {
    fn name(&self) -> &str {
        "CliError"
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

    fn cause(&self) -> Option<&Error> {
        match self.data.kind {
            CliErrorKind::InternalIoError(ref err) => Some(err as &Error),
            _ => None,
        }
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

impl FromError<io::IoError> for CliError {
    fn from_error(err: io::IoError, loc: Option<ErrorLocation>) -> CliError {
        CliError {
            data: box CommonErrorData {
                description: "an I/O error occurred",
                kind: CliErrorKind::InternalIoError(err),
                detail: None,
                location: loc,
            }
        }
    }
}

fn test_missing_item() -> Result<(), CliError> {
    fail!(CliErrorKind::NotFound, "The intended item does not exist.");
}

fn bar() -> Result<(), CliError> {
    try!(test_missing_item());
    fail!(CliErrorKind::NoPermission, "Access not possible");
}

fn read_first_line() -> Result<String, io::IoError> {
    let file = try!(io::File::open(&Path::new("/missing.txt")));
    let mut br = io::BufferedReader::new(file);
    br.read_line()
}

fn an_io_error() -> Result<(), CliError> {
    try!(read_first_line());
    Ok(())
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

    match an_io_error() {
        Err(e) => print_error_stack(&e),
        Ok(_) => {},
    }
}
