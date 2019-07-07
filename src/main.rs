extern crate chrono;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

mod db;

use db::{Data, Db, Entry, Predicate, PredicateType};
use std::fs::File;
use std::io;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;

// Read lines of a file into a vec
// Ignores lines beginning with '#'
pub fn read_file_to_vec(filename: &str) -> Vec<String> {
    let f = File::open(filename).unwrap();
    let file = BufReader::new(&f);
    let mut v: Vec<String> = vec![];
    for line in file.lines().skip(1) {
        if let Ok(line) = line {
            if let Some(c) = line.chars().nth(0) {
                if c != '#' {
                    v.push(line);
                }
            }
        }
    }
    v
}

fn load(dbname: &str, filename: &str) -> Db {
    if let Ok(db) = Db::load(dbname) {
        println!("Using existing db.");
        db
    } else {
        let mut db = Db::new(dbname);
        println!("Creating new db and loading from {}.", filename);

        let lines = read_file_to_vec(filename);
        for line in &lines {
            let mut split = line.split('|');
            if let Some(e) = split.next() {
                let mut entries: Vec<Entry> = vec![
                    Entry {
                        name: String::from("set"),
                        value: Db::db_string("es-en"),
                    },
                    Entry {
                        name: String::from("name"),
                        value: Db::db_string(e),
                    },
                ];

                for e in split {
                    entries.push(Entry {
                        name: String::from("value"),
                        value: Db::db_string(e),
                    });
                }
                let _id = db.add(entries);
            }
        }
        db
    }
}

fn find(db: &Db, name: &str, predicate_type: PredicateType) -> Vec<(String, String)> {
    // "set" needs to be at the end or search is very slow
    let predicates = vec![
        Predicate {
            predicate_type,
            entry: Entry::new_string("name", name),
        },
        Predicate::new_equal_string("set", "es-en"),
    ];

    let result = db.select(
        &predicates,
        vec![String::from("name"), String::from("value")],
    );
    result
        .iter()
        .map(|entry| (entry[0].value.clone(), entry[1].value.clone()))
        .filter_map(|e| match e {
            (Data::DbString(name), Data::DbString(value)) => Some((name, value)),
            _ => None,
        })
        .collect::<Vec<(String, String)>>()
}

fn present(result: Vec<(String, String)>) {
    if result.is_empty() {
        println!("No results.");
    } else {
        for line in &result {
            println!("{}: {}", line.0, line.1);
        }
    }
}

fn main() {
    let dbname = "default";
    let filename = "resources/es-en.txt";
    let db = load(dbname, filename);

    let mut input = String::new();
    print!("Enter search term: ");
    io::stdout().flush().unwrap();
    while let Ok(_bytes_read) = io::stdin().read_line(&mut input) {
        println!("Searching...");

        let result = find(&db, &input.trim(), PredicateType::Contains);
        if result.is_empty() {
            println!("\nSearch result empty.");
        } else {
            println!("\nFull matches:");
            present(result);
        }

        let result = find(&db, &input.trim(), PredicateType::StartsWith);
        if !result.is_empty() {
            println!("\nStarting with:");
            present(result);
        }

        let result = find(&db, &input.trim(), PredicateType::Equal);
        if !result.is_empty() {
            println!("\nEquals:");
            present(result);
        }

        println!("----------------------------");

        input.clear();

        print!("Enter search term: ");
        io::stdout().flush().unwrap();
    }

    println!("Saving database {}.", dbname);
    if let Ok(_result) = db.save() {
    } else {
        println!("Error saving database {}!", dbname);
    }
}

mod main {
    #[cfg(test)]
    use super::*;
    #[test]
    fn load_and_filter() {
        let dbname = "test-sample";
        let filename = "resources/es-en-sample.txt";
        let db = load(dbname, filename);

        let values = find(&db, "coche", PredicateType::Equal);
        assert_eq!(values.len(), 1);
        assert_eq!(values[0].0, "coche");
        assert_eq!(values[0].1, "car");

        let values = find(&db, "coche", PredicateType::StartsWith);
        assert_eq!(values.len(), 5);
        assert_eq!(values[2].0, "coche eléctrico");
        assert_eq!(values[2].1, "electric car");

        let values = find(&db, "coche", PredicateType::Contains);
        assert_eq!(values.len(), 6);
        assert_eq!(values[5].0, "lavacoches");
        assert_eq!(values[5].1, "carwash");
    }
}
