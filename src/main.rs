extern crate chrono;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

mod db;

use db::Data::DbString;
use db::{Db, Entry};
use std::fs::File;
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

fn main() {
    let dbname = "default";
    let filename = "resources/es-en.txt";
    let db = if let Ok(db) = Db::load(dbname) {
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
    };

    let coche_row_id = db
        .rows
        .iter()
        .filter(|row| row.entry.name == "name")
        .filter(|row| DbString(String::from("coche")) == row.entry.value)
        .map(|row| row.row_id)
        .next();
    println!("'coche row_id': {:?}", coche_row_id);

    for value in db
        .rows
        .iter()
        .filter(|row| Some(row.row_id) == coche_row_id)
        .filter(|row| row.entry.name == "value")
        .map(|row| row.entry.value.clone())
    {
        println!("'coche value': {:?}", value);
    }

    println!("Saving database {}.", dbname);
    if let Ok(_result) = db.save() {
    } else {
        println!("Error saving database {}!", dbname);
    }
}

mod main {
    #[test]
    fn fun() {}
}
