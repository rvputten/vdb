extern crate chrono;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

mod db;

use db::{Data, Db, Entry, Predicate, PredicateType, RowId};
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

fn load(db_vocabulary_name: &str, db_personal_name: &str, vocabulary_filename: &str) -> (Db, Db) {
    let db_vocabulary = if let Ok(db) = Db::load(db_vocabulary_name) {
        println!("Using existing db for vocabulary.");
        db
    } else {
        let mut db = Db::new(db_vocabulary_name);
        println!(
            "Creating new db for vocabulary and loading from {}.",
            vocabulary_filename
        );

        let lines = read_file_to_vec(vocabulary_filename);
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
    let db_personal = if let Ok(db) = Db::load(db_personal_name) {
        println!("Using existing db for personal dictionary.");
        db
    } else {
        let mut db = Db::new(db_personal_name);
        println!("Creating new db for personal dictionary.");
        db
    };
    let db_personal = Db::new(db_personal_name);
    (db_vocabulary, db_personal)
}

#[cfg(test)]
fn find(db: &Db, name: &str, predicate_type: PredicateType) -> Vec<(String, String)> {
    let row_ids = find_row_ids(db, name, predicate_type, None);
    find_row_ids_to_columns(db, &row_ids)
}

fn find_row_ids(
    db: &Db,
    name: &str,
    predicate_type: PredicateType,
    max_results: Option<usize>,
) -> Vec<RowId> {
    // "set" needs to be at the end or search is very slow (needs high selectivity)
    let predicates = vec![
        Predicate {
            predicate_type,
            entry: Entry::new_string("name", name),
        },
        Predicate::new_equal_string("set", "es-en"),
    ];

    db.select_row_ids(&predicates, max_results)
}

fn find_row_ids_to_columns(db: &Db, row_ids: &[RowId]) -> Vec<(String, String)> {
    let columns = vec![String::from("name"), String::from("value")];
    let result = db.columns_from_row_ids(row_ids, columns);
    result
        .iter()
        .map(|entry| (entry[0].value.clone(), entry[1].value.clone()))
        .filter_map(|e| match e {
            (Data::DbString(name), Data::DbString(value)) => Some((name, value)),
            _ => None,
        })
        .collect::<Vec<(String, String)>>()
}

fn present(db: &Db, row_ids: &[RowId], max_message: bool) {
    for line in &find_row_ids_to_columns(db, row_ids) {
        println!("{}: {}", line.0, line.1);
    }
    if max_message {
        println!();
        println!("Limited number of rows shown.");
    }
}

fn minus(left: &[RowId], right: &[RowId]) -> Vec<RowId> {
    left.iter()
        .filter_map(|&x| if right.contains(&x) { None } else { Some(x) })
        .collect::<Vec<RowId>>()
}

fn main() {
    let db_vocabulary_name = "vocabulary";
    let db_personal_name = "personal";
    let filename = "resources/es-en.txt";

    let (db_vocabulary, db_personal) = load(db_vocabulary_name, db_personal_name, filename);

    main_loop(&db_vocabulary);

    save(&db_personal, db_personal_name);
}

fn main_loop(db_vocabulary: &Db) {
    let mut input = String::new();
    let max_results: usize = 100;

    print!("Enter search term: ");
    io::stdout().flush().unwrap();
    while let Ok(_bytes_read) = io::stdin().read_line(&mut input) {
        let trimmed = input.trim();
        if trimmed == "" {
            break;
        }

        let rows_equal = find_row_ids(
            &db_vocabulary,
            &trimmed,
            PredicateType::Equal,
            Some(max_results),
        );
        let number_matches_equal = rows_equal.len();

        let rows_starts_with_full = find_row_ids(
            &db_vocabulary,
            &trimmed,
            PredicateType::StartsWith,
            Some(max_results),
        );
        let rows_starts_with = minus(&rows_starts_with_full, &rows_equal);
        let number_matches_starts_with = rows_starts_with_full.len();

        if number_matches_starts_with < max_results {
            let rows_contains_full = find_row_ids(
                &db_vocabulary,
                &trimmed,
                PredicateType::Contains,
                Some(max_results),
            );
            let number_matches_contains = rows_contains_full.len();

            if number_matches_contains == 0 {
                println!("\nSearch result empty.");
            } else {
                let rows_contains = minus(&rows_contains_full, &rows_starts_with_full);
                println!("\nFull matches:");
                present(
                    &db_vocabulary,
                    &rows_contains,
                    number_matches_contains == max_results,
                );
            }
        }

        if number_matches_starts_with > 0 {
            println!("\nStarting with:");
            present(
                &db_vocabulary,
                &rows_starts_with,
                number_matches_starts_with == max_results,
            );
        }

        if number_matches_equal > 0 {
            println!("\nEquals:");
            present(
                &db_vocabulary,
                &rows_equal,
                number_matches_equal == max_results,
            );
        }

        println!("----------------------------");

        input.clear();

        print!("Enter search term: ");
        io::stdout().flush().unwrap();
    }
}
fn save(db: &Db, db_name: &str) {
    println!("Saving database {}.", db_name);
    if let Ok(_result) = db.save() {
    } else {
        println!("Error saving database {}!", db_name);
    }
}

mod main {
    #[cfg(test)]
    use super::*;

    #[test]
    fn minus2() {
        let rows1 = vec![RowId(1), RowId(2), RowId(4), RowId(8), RowId(6)];
        let rows2 = vec![RowId(2), RowId(4)];
        let result = vec![RowId(1), RowId(8), RowId(6)];
        assert_eq!(minus(&rows1, &rows2), result);
    }

    #[test]
    fn load_and_filter() {
        let dbname = "test-sample";
        let filename = "resources/es-en-sample.txt";
        let (db, _) = load(dbname, "dummy", filename);

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
