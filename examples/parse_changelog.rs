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
