[![Crates.io]](https://crates.io/crates/poreader)
[![Docs.rs](https://img.shields.io/docsrs/poreader?style=for-the-badge)](https://docs.rs/poreader/)
[![CircleCI]](https://circleci.com/gh/corebreaker/poreader/tree/main)
[![Coverage Status]](https://coveralls.io/github/corebreaker/poreader?branch=main)

# `poreader`

Rust library for reading translation catalogs in Uniforum/Gettext PO.
Similar to the [translate.storage] package in Python [Translate Toolkit].

Only PO and Xliff are planned to be supported. For anything else, just convert it with [Translate Toolkit].
There is no point in replacing that excellent library;
the main reason for Rust parser and writer is to them as part of build
process of Rust programs, especially in procedural macros, which need to be written in Rust.

## Documentation

On [Docs.rs].

## Installation

It uses [Cargo], Rust's package manager. You can depend on this library by adding `poreader` to your Cargo dependencies:

```toml
[dependencies]
poreader = "1.1"
```

Or, to use the Git repo directly:
```toml
[dependencies.poreader]
git = "https://github.com/corebreaker/poreader.git"
```

## How to use

Start by creating a PO reader from a new PO parser, then iterate on the reader:
```rust
use poreader::PoParser;

use std::{env::args, fs::File, io::{Result, Error, ErrorKind}};

struct NoArg;
impl std::error::Error for NoArg {}

impl std::fmt::Display for NoArg {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result { std::fmt::Debug::fmt(self, f) }
}

impl std::fmt::Debug for NoArg {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result { write!(f, "No file specified") }
}

fn main() -> Result<()> {
    // Filename
    let filename = match args().skip(1).next() {
        Some(v) => v,
        None => { return Err(Error::new(ErrorKind::Other, NoArg)); }
    };

    // Open a file
    let file = File::open(filename)?;

    // Create PO parser
    let parser = PoParser::new();
    
    // Create PO reader
    let reader = parser.parse(file)?;

    // Read PO file by iterating on units
    for unit in reader {
        let unit = unit?;

        // Show `msgid`
        println!(" - {}", unit.message().get_id())
    }

    Ok(())
}
```

# Status of the project

The project works for instance.

Future changes is directely related to the change of the format of the PO file.

However, it would be updated with a need from contributors.
You can post an issue or a pull request if you see something that can be improved.


[Cargo]: http://crates.io
[Docs.rs]: https://docs.rs/poreader/
[translate.storage]: http://docs.translatehouse.org/projects/translate-toolkit/en/latest/api/storage.html
[Translate Toolkit]: https://pypi.org/project/translate-toolkit/
[Crates.io]: https://img.shields.io/crates/v/poreader?style=for-the-badge
[CircleCI]: https://img.shields.io/circleci/build/github/corebreaker/poreader/main?style=for-the-badge
[Coverage Status]: https://img.shields.io/coveralls/github/corebreaker/poreader?style=for-the-badge
