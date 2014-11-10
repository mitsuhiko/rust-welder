# Rust Welder

This repository contains experiments to implement a error handling concept for
Rust.  Presently it's not quite clear what the correct approach is.

## Requirements

* Error representation in the result is word size (or maybe two word size).
  There is currently a trend for way too large errors (for instance `IoResult`)
  which really is not very good.
* Errors should contain location information about where the occurred.  This
  needs to happen through macros and not runtime support because we're in a
  systems language :)
* Errors need to be able to wrap each other and convert between each other.
* It should be possible to mask errors and to respond to them.  For this matter
  it's interesting to have a error kind associated that a caller can test for.
