# Claybrick

 [![test & clippy](https://gitlab.com/weichweich/claybrick/badges/main/pipeline.svg)](https://gitlab.com/weichweich/claybrick/-/commits/main) 

> Just for fun and learning.

A PDF library that (for now) only reads PDFs.
Short term goal is to  support rearranging pages in a PDF document.

## Examples

* `cargo run --example catalog -- --help` print the catalog of a PDF
* `cargo run --example trace --features trace -- --help` parse a PDF and output huge amounts of debug logs
* `cargo run --example xref -- --help` print the xref section

## Design

The `claybbrick` project is split into 3 parts pdf, parse, encode.
The `pdf` module contains structs, enums and primitives that make it possible to represent a PDF file in memory.
It should have no dependencies to the `parse` and `simple_encode` modules, since the parsing and encoding of a PDF should be something that can be replaces by a better implementation later.

## Other PDF libraries

There are a few Rust PDF libraries out there.

* [pdf-rs/pdf](https://github.com/pdf-rs/pdf)
* [lopdf](https://github.com/J-F-Liu/lopdf)
* [murtyjones/purdy](https://github.com/murtyjones/purdy)
