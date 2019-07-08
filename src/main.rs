extern crate chrono;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

mod db;

pub use db::{Data, Db, Entry, Predicate, PredicateType, RowId};
use std::fs::File;
use std::io;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;

/// Read lines of a file into a Vec<String>.
/// Ignores lines beginning with '#'.
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
        let db = Db::new(db_personal_name);
        println!("Creating new db for personal dictionary.");
        db
    };
    (db_vocabulary, db_personal)
}

#[cfg(test)]
fn find(db: &mut Db, name: &str, predicate_type: PredicateType) -> Vec<(usize, String, String)> {
    let row_ids = find_row_ids(db, name, predicate_type, None);
    add_numbers(db, &row_ids, 0);
    find_row_ids_to_entries(db, &row_ids)
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

fn find_row_ids_to_entries(db: &Db, row_ids: &[RowId]) -> Vec<(usize, String, String)> {
    let entries = vec![
        String::from("search_index"),
        String::from("name"),
        String::from("value"),
    ];
    let result = db.entries_from_row_ids(row_ids, entries);
    result
        .iter()
        .map(|entries| {
            (
                Entry::get_by_name(entries, "search_index"),
                Entry::get_by_name(entries, "name"),
                Entry::get_by_name(entries, "value"),
            )
        })
        .filter_map(|e| match e {
            (Some(index), Some(name), Some(value)) => Some((index.value, name.value, value.value)),
            _ => None,
        })
        .filter_map(|e| match e {
            (Data::DbInt(index), Data::DbString(name), Data::DbString(value)) => {
                Some((index as usize, name, value))
            }
            _ => None,
        })
        .collect::<Vec<(usize, String, String)>>()
}

fn present(db: &Db, row_ids: &[RowId], max_message: bool) {
    for line in &find_row_ids_to_entries(db, row_ids) {
        println!("{}) {}: {}", line.0, line.1, line.2);
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

    let (mut db_vocabulary, mut db_personal) = load(db_vocabulary_name, db_personal_name, filename);

    main_loop(&mut db_vocabulary, &mut db_personal);

    save(&db_personal, db_personal_name);
}

fn main_loop(db_vocabulary: &mut Db, db_personal: &mut Db) {
    let mut input = String::new();
    let max_results: usize = 100;

    display_personal_db(db_personal, 100);

    print!("Enter search term: ");
    io::stdout().flush().unwrap();
    while let Ok(_bytes_read) = io::stdin().read_line(&mut input) {
        let trimmed = input.trim();
        if trimmed == "" {
            break;
        }

        if let Ok(number) = trimmed.parse::<usize>() {
            add_to_personal_db(db_vocabulary, db_personal, number);
            display_personal_db(db_personal, 1);
        } else if trimmed == "p" {
            display_personal_db(db_personal, 100);
        } else {
            db_vocabulary.delete_entry_all("search_index");
            find_and_display(db_vocabulary, trimmed, max_results);
            display_personal_db(db_personal, 7);
        }

        input.clear();

        print!("Enter search term or enter number to save in personal dictionary: ");
        io::stdout().flush().unwrap();
    }
}

fn display_personal_db(db_personal: &mut Db, max_rows: usize) {
    println!();
    println!("Personal dictionary:");

    let row_ids = db_personal.last_n_rows(max_rows);
    for row_id in row_ids {
        if let (Some(name), Some(value)) = (
            db_personal.get_entry(row_id, "name"),
            db_personal.get_entry(row_id, "value"),
        ) {
            match (&name.value, &value.value) {
                (Data::DbString(name), Data::DbString(value)) => {
                    println!("{}: {}", name, value);
                }
                _ => panic!("name, value not found"),
            };
        }
    }
    println!();
}

fn add_to_personal_db(db_vocabulary: &mut Db, db_personal: &mut Db, number: usize) {
    let predicates = vec![Predicate::new_equal_int("search_index", number as i32)];
    let row_ids = db_vocabulary.select_row_ids(&predicates, Some(1));
    if !row_ids.is_empty() {
        let name = db_vocabulary.get_entry(row_ids[0], "name");
        let value = db_vocabulary.get_entry(row_ids[0], "value");
        match (name, value) {
            (Some(name), Some(value)) => db_personal.add(vec![
                Entry {
                    name: String::from("name"),
                    value: name.value.clone(),
                },
                Entry {
                    name: String::from("value"),
                    value: value.value.clone(),
                },
            ]),
            _ => panic!("Couldn't find entries"),
        };
    } else {
        println!("No search result number {} found.", number);
    }
}

fn find_and_display(db: &mut Db, search_term: &str, max_results: usize) {
    let rows_equal = find_row_ids(&db, search_term, PredicateType::Equal, Some(max_results));
    let number_matches_equal = rows_equal.len();

    let rows_starts_with_full = find_row_ids(
        &db,
        search_term,
        PredicateType::StartsWith,
        Some(max_results),
    );
    let rows_starts_with = minus(&rows_starts_with_full, &rows_equal);
    let number_matches_starts_with = rows_starts_with_full.len();

    if number_matches_starts_with < max_results {
        let rows_contains_full =
            find_row_ids(&db, search_term, PredicateType::Contains, Some(max_results));
        let number_matches_contains = rows_contains_full.len();

        if number_matches_contains == 0 {
            println!("\nSearch result empty.");
        } else {
            let rows_contains = minus(&rows_contains_full, &rows_starts_with_full);
            if !rows_contains.is_empty() {
                println!("\nFull matches:");
                add_numbers(db, &rows_contains, number_matches_starts_with);
                present(&db, &rows_contains, number_matches_contains == max_results);
            }
        }
    }

    if number_matches_starts_with > 0 && !rows_starts_with.is_empty() {
        println!("\nStarting with:");
        add_numbers(db, &rows_starts_with, number_matches_equal);
        present(
            &db,
            &rows_starts_with,
            number_matches_starts_with == max_results,
        );
    }

    if number_matches_equal > 0 {
        println!("\nEquals:");
        add_numbers(db, &rows_equal, 0);
        present(&db, &rows_equal, number_matches_equal == max_results);
    }

    println!("----------------------------");
}

fn add_numbers(db: &mut Db, row_ids: &[RowId], offset: usize) {
    let count = row_ids.len();
    let reverse_numbers = (0..count).map(|n| count - n + offset);
    for (row_id, index) in row_ids.iter().zip(reverse_numbers) {
        let row_id: RowId = *row_id;
        db.add_entry(
            row_id,
            Entry {
                name: String::from("search_index"),
                value: Db::db_int(index as i32),
            },
        );
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
        let (mut db, _) = load(dbname, "dummy", filename);

        let values = find(&mut db, "coche", PredicateType::Equal);
        assert_eq!(values.len(), 1);
        assert_eq!(values[0].1, "coche");
        assert_eq!(values[0].2, "car");

        let values = find(&mut db, "coche", PredicateType::StartsWith);
        assert_eq!(values.len(), 5);
        assert_eq!(values[2].1, "coche eléctrico");
        assert_eq!(values[2].2, "electric car");

        let values = find(&mut db, "coche", PredicateType::Contains);
        assert_eq!(values.len(), 6);
        assert_eq!(values[5].1, "lavacoches");
        assert_eq!(values[5].2, "carwash");
    }
}
