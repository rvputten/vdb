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

/// Adds a single word with translation to the database. Multiple translations for a word will be
/// appended to the same row. Duplicates are filtered.
fn add_word(db: &mut Db, word: &str, translations: &[String]) {
    if let Some(row_id) = db.find_first_row_id_by_value("name", &Db::db_string(word)) {
        let entries = db.entries_from_row_ids(&[row_id], vec![String::from("value")]);
        for value in translations {
            if let Some(_entry) = entries
                .iter()
                .flatten()
                .find(|entry| entry.value == Db::db_string(value))
            {
            } else {
                db.add_entry(
                    row_id,
                    Entry {
                        name: String::from("value"),
                        value: Db::db_string(value),
                    },
                );
            }
        }
    } else {
        let mut entries: Vec<Entry> = vec![Entry {
            name: String::from("name"),
            value: Db::db_string(word),
        }];

        for value in translations {
            entries.push(Entry {
                name: String::from("value"),
                value: Db::db_string(value),
            });
        }
        let _id = db.add(entries);
    }
}

/// Load vocabulary from file. The file format is (example)
/// ```c
/// nube|A cloud|A mutitude, or crowd, of people
/// ```
/// i. e. the word is in the first column, then multiple translations
fn load(
    db_vocabulary_name: &str,
    db_personal_name: &str,
    vocabulary_filename: &str,
    reload: bool,
) -> (Db, Db) {
    let db_vocabulary = match (reload, Db::load(db_vocabulary_name)) {
        (false, Ok(db)) => {
            println!("Using existing db for vocabulary.");
            db
        }
        _ => {
            let mut db = Db::new(db_vocabulary_name);
            println!(
                "Creating new db for vocabulary and loading from {}.",
                vocabulary_filename
            );

            let lines = read_file_to_vec(vocabulary_filename);
            let n = lines.len();
            for (i, line) in lines.iter().enumerate() {
                print!("\r{}/{}", i, n);

                let mut split = line.split('|');
                if let Some(name) = split.next() {
                    let values = split.map(|s| s.to_string()).collect::<Vec<String>>();
                    add_word(&mut db, name, &values);
                }
            }
            println!("\r{} lines loaded.", n);
            save(&db, db_vocabulary_name);
            db
        }
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
fn find(
    db: &mut Db,
    name: &str,
    predicate_type: PredicateType,
) -> Vec<(usize, String, Vec<String>)> {
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
    let predicates = vec![Predicate {
        predicate_type,
        entry: Entry::new_string("name", name),
    }];

    db.select_row_ids(&predicates, max_results)
}

fn find_row_ids_to_entries(db: &Db, row_ids: &[RowId]) -> Vec<(usize, String, Vec<String>)> {
    let mut result: Vec<(usize, String, Vec<String>)> = vec![];

    let names = vec![
        String::from("search_index"),
        String::from("name"),
        String::from("value"),
    ];
    let rows = db.entries_from_row_ids(row_ids, names);
    for row in rows {
        let mut index: usize = 0;
        let mut name: Option<String> = None;
        let mut values: Vec<String> = vec![];
        for entry in row {
            match (entry.name.as_str(), &entry.value) {
                ("search_index", Data::DbInt(n)) => index = *n as usize,
                ("name", Data::DbString(s)) => name = Some(s.to_string()),
                ("value", Data::DbString(s)) => values.push(s.to_string()),
                _ => panic!("Unknown entry {:?}", entry),
            }
        }
        if let Some(name) = name.clone() {
            result.push((index, name, values));
        }
    }
    result
}

fn present(db: &Db, row_ids: &[RowId], max_message: bool) {
    for line in &find_row_ids_to_entries(db, row_ids) {
        println!("{}) {}: {}", line.0, line.1, line.2.join("; "));
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

    let (mut db_vocabulary, mut db_personal) =
        load(db_vocabulary_name, db_personal_name, filename, false);

    main_loop(&mut db_vocabulary, &mut db_personal);

    save(&db_personal, db_personal_name);
}

fn main_loop(db_vocabulary: &mut Db, db_personal: &mut Db) {
    let mut input = String::new();
    let max_results: usize = 100;

    display_personal_db(db_personal, 100, true);

    print!("Enter search term: ");
    io::stdout().flush().unwrap();
    while let Ok(_bytes_read) = io::stdin().read_line(&mut input) {
        let trimmed = input.trim();
        if trimmed == "" {
            break;
        }

        if let Ok(number) = trimmed.parse::<usize>() {
            add_to_personal_db(db_vocabulary, db_personal, number);
        } else if trimmed == "p" {
            display_personal_db(db_personal, 100, true);
        } else {
            db_vocabulary.delete_entry_all("search_index");
            find_and_display(db_vocabulary, trimmed, max_results);
            display_personal_db(db_personal, 7, false);
        }

        input.clear();

        println!(
            "================================================================================"
        );
        println!();
        print!("Enter search term or enter number to save in personal dictionary: ");
        io::stdout().flush().unwrap();
    }
}

fn display_personal_db(db_personal: &mut Db, max_rows: usize, sort: bool) {
    println!();
    println!("Personal dictionary:");

    let row_ids = db_personal.last_n_rows(max_rows);
    let mut results = find_row_ids_to_entries(db_personal, &row_ids);
    if sort {
        results.sort_by(
            |(_index_a, name_a, _value_a), (_index_b, name_b, _value_b)| name_a.cmp(name_b),
        );
    }
    for (_index, name, value) in results {
        println!("{}: {}", name, value.join("; "));
    }
    println!();
}

fn add_to_personal_db(db_vocabulary: &mut Db, db_personal: &mut Db, number: usize) {
    let predicates = vec![Predicate::new_equal_int("search_index", number as i32)];
    let row_ids = db_vocabulary.select_row_ids(&predicates, Some(1));
    for (_index, name, value) in find_row_ids_to_entries(db_vocabulary, &row_ids) {
        println!("Adding {}: {}", name, value.join("; "));
        add_word(db_personal, &name, &value);
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
        db.add_or_update_entry(
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
        let (mut db, _) = load(dbname, "dummy", filename, true);

        let row_ids = db.select_row_ids(&[], None);

        let values = find(&mut db, "coche", PredicateType::Equal);
        assert_eq!(values.len(), 1);
        assert_eq!(values[0].1, "coche");
        assert_eq!(values[0].2[0], "car");

        let values = find(&mut db, "coche", PredicateType::StartsWith);
        assert_eq!(values.len(), 4);
        assert_eq!(values[2].1, "coche eléctrico");
        assert_eq!(values[2].2[0], "electric car");

        let values = find(&mut db, "coche", PredicateType::Contains);
        assert_eq!(values.len(), 5);
        assert_eq!(values[4].1, "lavacoches");
        assert_eq!(values[4].2[0], "carwash");
    }
}
