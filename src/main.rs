extern crate chrono;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

mod db;

use db::{Data, Db, Entry};
use std::fs::File;
use std::io;
use std::io::BufRead;
use std::io::BufReader;

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

fn find(db: &Db, name: &str) -> Vec<String> {
    // "set" needs to be at the end or search is very slow
    let predicates = vec![
        Entry {
            name: String::from("name"),
            value: Db::db_string(name),
        },
        Entry {
            name: String::from("set"),
            value: Db::db_string("es-en"),
        },
    ];
    let result = db.select(&predicates, vec![String::from("value")]);
    result
        .iter()
        .map(|entry| entry[0].value.clone())
        .filter_map(|e| match e {
            Data::DbString(v) => Some(v),
            _ => None,
        })
        .collect::<Vec<String>>()
}

fn main() {
    let dbname = "default";
    let filename = "resources/es-en.txt";
    let db = load(dbname, filename);

    let mut input = String::new();
    while let Ok(_bytes_read) = io::stdin().read_line(&mut input) {
        println!("{:?}", find(&db, &input.trim()));
        input.clear();
    }

    println!("Saving database {}.", dbname);
    if let Ok(_result) = db.save() {
    } else {
        println!("Error saving database {}!", dbname);
    }
}

mod main {
    use super::*;
    #[test]
    fn load_and_filter() {
        let dbname = "test-sample";
        let filename = "resources/es-en-sample.txt";
        let db = load(dbname, filename);

        let values = find(&db, "coche");

        assert_eq!(values.len(), 1);
        assert_eq!(values[0], "car");
    }
}
