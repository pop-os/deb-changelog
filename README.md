# debian-changelog

Rust crate for efficient parsing and generation of debian changelogs. Allocations are avoided where
possible, and the Entry structure is reused upon each iteration of a changelog.

> Experimental async/await support may also be enabled for appending entries to an existing
changelog. Using the `tokio-async` feature enables `tokio`-based async runtime support, whereas
`std-async` enables `async-std`-based runtime support.
>
> Requires Rust 1.39.0 to build with async support.

```rust
use debian_changelog::Entry;
use std::{env, fs};

fn main() {
    let changelog = env::args().skip(1).next().unwrap();
    let changelog = fs::read_to_string(&changelog).unwrap();

    let mut entry = Entry::default();
    let mut iterator = entry.iter_from(changelog.as_str());

    while let Some(result) = iterator.next() {
        match result {
            Ok(entry) => {
                println!("Debug: {:#?}", entry);
                println!("Format: {}", entry);
            }
            Err(why) => {
                eprintln!("{}", why);
                break;
            }
        }
    }
}

```

## License

Licensed under the Mozilla Public License, Version 2.0. ([LICENSE](LICENSE) or https://www.mozilla.org/en-US/MPL/2.0/)

### Contribution

Any contribution intentionally submitted for inclusion in the work by you, shall be licensed under the MPL-2.0.
