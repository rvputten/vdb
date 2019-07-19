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

#[derive(Debug)]
struct DictEntry {
    word: Option<String>,
    translations: Vec<String>,
    conjugations: Vec<String>,
    index: Option<usize>,
    add_date: Option<Data>,
    add_counter: Option<usize>,
}

/// Read lines of a file into a Vec<String>.
/// Ignores lines beginning with '#'.
pub fn read_file_to_vec(filename: &str) -> Vec<String> {
    let f = File::open(filename).unwrap();
    let file = BufReader::new(&f);
    file.lines()
        .filter_map(|line| {
            if let Ok(line) = line {
                Some(line)
            } else {
                None
            }
        })
        .collect::<Vec<String>>()
}

/// The usage counter lets the user track how many times the word was added to the database
fn update_counter(db: &mut Db, row_id: RowId) {
    let counter_name = "add_counter";
    let new_counter = if let Some(entry) = db.get_first_entry_mut(row_id, counter_name) {
        if let Data::DbInt(counter) = entry.value {
            counter + 1
        } else {
            1
        }
    } else {
        1
    };
    db.add_or_update_entry(
        row_id,
        Entry {
            name: counter_name.to_string(),
            value: Data::DbInt(new_counter),
        },
    );
}

/// Track last time the entry was added
fn update_date(db: &mut Db, row_id: RowId) {
    let date_name = "add_date";
    db.add_or_update_entry(
        row_id,
        Entry {
            name: date_name.to_string(),
            value: Data::now(),
        },
    );
}

/// Adds a single word with translation to the database. Multiple translations for a word will be
/// appended to the same row. Duplicates are filtered.
fn add_word(db: &mut Db, word: &str, translations: &[String], add_time: bool, update_count: bool) {
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

    if add_time || update_count {
        if let Some(row_id) = db.find_first_row_id_by_value("name", &Db::db_string(word)) {
            if update_count {
                update_counter(db, row_id);
            }
            if add_time {
                update_date(db, row_id);
            }
        }
    }
}

/// Load irregular verbs from file.
/// Parsing is adapted to the given source file "resources/irregular_verbs/irregular_verbs.txt".
/// Of the 1321 verbs in the file, 603 do not have a corresponding entry in the vocabulary file
/// "es-en.txt". However, they seem to all be very rare words, so we ignore them.
fn load_irregular_verbs(db_vocabulary: &mut Db, db_vocabulary_name: &str, filename: &str) {
    if let Some(_row_id) = db_vocabulary.find_first_row_id_by_name("conjugation") {
        println!("Conjugations already loaded, skipping load.");
    } else {
        let lines = read_file_to_vec(filename);
        let mut existing = 0;
        let mut not_existing = 0;

        println!("Mapping conjugations (may take a while in debug mode)...");

        for line in lines.iter() {
            let mut split_line = line.split(':');
            // The source file always has two entries per line
            let infinitive = split_line.next().unwrap();

            // This operation is currently slow (~5 secs in debug mode, < 1 sec in release mode) and needs to be sped up
            if let Some(row_id) =
                db_vocabulary.find_first_row_id_by_value("name", &Db::db_string(infinitive))
            {
                existing += 1;

                let conjugations = split_line.next().unwrap();
                for s in conjugations.split(',') {
                    let t = s.split('|').collect::<Vec<&str>>();
                    // There are a few lines without form indicator in the source file. len() is always either 1 or 2.
                    let (_form_indicator, conjugation) = if t.len() == 2 {
                        (t[0], t[1])
                    } else {
                        ("none", t[0])
                    };
                    db_vocabulary.add_entry(row_id, Entry::new_string("conjugation", conjugation));
                    //println!("{}|{}|{}", infinitive, form_indicator, conjugation);
                }
            } else {
                not_existing += 1;
            }
            print!(
                "\rAdded {} and rejected {} out of {}",
                existing,
                not_existing,
                existing + not_existing
            );
        }
        println!();
        println!("Saving with conjugations...");
        save(&db_vocabulary, db_vocabulary_name);
    }
}

/// Load vocabulary from file. The file format is (example)
/// ```c
/// nube|A cloud|A mutitude, or crowd, of people
/// ```
/// i. e. the word is in the first column, then multiple translations
fn load_dictionary(
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
                    add_word(&mut db, name, &values, false, false);
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
fn find(db: &mut Db, name: &str, predicate_type: PredicateType) -> Vec<DictEntry> {
    let row_ids = find_row_ids(db, "name", name, predicate_type, None);
    add_numbers(db, &row_ids, 0);
    find_row_ids_to_entries(db, &row_ids)
}

fn find_row_ids(
    db: &Db,
    column: &str,
    name: &str,
    predicate_type: PredicateType,
    max_results: Option<usize>,
) -> Vec<RowId> {
    // "set" needs to be at the end or search is very slow (needs high selectivity)
    let predicates = vec![Predicate {
        predicate_type,
        entry: Entry::new_string(column, name),
    }];

    db.select_row_ids(&predicates, max_results)
}

fn find_row_ids_to_entries(db: &Db, row_ids: &[RowId]) -> Vec<DictEntry> {
    let mut result: Vec<DictEntry> = vec![];

    let names = vec![
        String::from("search_index"),
        String::from("name"),
        String::from("value"),
        String::from("conjugation"),
        String::from("add_date"),
        String::from("add_counter"),
    ];
    let rows = db.entries_from_row_ids(row_ids, names);
    for row in rows {
        let mut dict_entry = DictEntry {
            index: None,
            word: None,
            translations: vec![],
            conjugations: vec![],
            add_date: None,
            add_counter: None,
        };
        for entry in row {
            match (entry.name.as_str(), &entry.value) {
                ("search_index", Data::DbInt(n)) => dict_entry.index = Some(*n as usize),
                ("name", Data::DbString(s)) => dict_entry.word = Some(s.to_string()),
                ("value", Data::DbString(s)) => dict_entry.translations.push(s.to_string()),
                ("conjugation", Data::DbString(s)) => dict_entry.conjugations.push(s.to_string()),
                ("add_date", date) => dict_entry.add_date = Some(date.clone()),
                ("add_counter", Data::DbInt(counter)) => {
                    dict_entry.add_counter = Some(*counter as usize)
                }
                _ => panic!("Unknown entry {:?}", entry),
            }
        }
        if let Some(_word) = &dict_entry.word {
            result.push(dict_entry);
        }
    }
    result
}

fn present(db: &Db, row_ids: &[RowId], max_message: bool) {
    for dict_entry in &find_row_ids_to_entries(db, row_ids) {
        if let Some(word) = &dict_entry.word {
            let index = if let Some(index) = dict_entry.index {
                format!("{})", index)
            } else {
                "".to_string()
            };
            println!("{} {}: {}", index, word, dict_entry.translations[0]);

            let spaces = " ".repeat(index.len());
            for translation in dict_entry.translations.iter().skip(1) {
                println!("{} {}: {}", spaces, word, translation);
            }
        }
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
    let filename = "resources/es-en/es-en.txt";

    let (mut db_vocabulary, mut db_personal) =
        load_dictionary(db_vocabulary_name, db_personal_name, filename, false);
    load_irregular_verbs(
        &mut db_vocabulary,
        db_vocabulary_name,
        "resources/irregular_verbs/irregular_verbs.txt",
    );

    main_loop(&mut db_vocabulary, &mut db_personal);

    save(&db_personal, db_personal_name);
}

fn main_loop(db_vocabulary: &mut Db, db_personal: &mut Db) {
    let mut input = String::new();
    let max_results: usize = 100;

    display_personal_db(db_personal, 100, false, None);

    print!("Enter search term: ");
    io::stdout().flush().unwrap();
    while let Ok(_bytes_read) = io::stdin().read_line(&mut input) {
        let trimmed = input.trim();
        if trimmed == "" {
            break;
        }

        if let Ok(number) = trimmed.parse::<usize>() {
            add_to_personal_db(db_vocabulary, db_personal, number);
            display_personal_db(db_personal, 1, true, None);
        } else {
            let mut words = trimmed.split_whitespace();
            if words.next() == Some("p") {
                display_personal_db(db_personal, 100, false, words.next());
            } else {
                db_vocabulary.delete_entry_all("search_index");
                display_personal_db(db_personal, 30, true, None);
                find_and_display(db_vocabulary, trimmed, max_results);
            }
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

fn sort_db(entries: &mut Vec<DictEntry>) {
    entries.sort_by(
        |entry_a, entry_b| match (&entry_a.add_date, &entry_b.add_date) {
            (Some(Data::DbDateTime(date_a)), Some(Data::DbDateTime(date_b))) => date_a.cmp(&date_b),
            (Some(Data::DbDateTime(_date_a)), _) => std::cmp::Ordering::Greater,
            (_, Some(Data::DbDateTime(_date_b))) => std::cmp::Ordering::Less,
            _ => entry_a.word.cmp(&entry_b.word),
        },
    );
}

fn display_personal_db(
    db_personal: &mut Db,
    max_rows: usize,
    compact: bool,
    starts_with: Option<&str>,
) {
    println!();
    println!("Personal dictionary (last {} entries):", max_rows);

    //let row_ids = db_personal.last_n_rows(max_rows);
    let row_ids = if let Some(starts_with) = starts_with {
        db_personal.select_row_ids(&[Predicate::new_starts_with("name", starts_with)], None)
    } else {
        db_personal.enumerate_row_ids()
    };
    let mut results = find_row_ids_to_entries(db_personal, &row_ids);
    sort_db(&mut results);

    if compact {
        let row_count = results.len();
        let start_row = if row_count > max_rows {
            row_count - max_rows
        } else {
            0
        };
        for (add_counter, add_date, word, translations) in
            results.iter().skip(start_row).filter_map(|entry| {
                if let (Some(add_counter), Some(add_date), Some(word)) = (
                    entry.add_counter,
                    entry.add_date.clone(),
                    entry.word.clone(),
                ) {
                    Some((add_counter, add_date, word, entry.translations.clone()))
                } else {
                    None
                }
            })
        {
            println!(
                "{} {}: {} ({})",
                add_date,
                word,
                translations.join("; "),
                add_counter
            );
        }
    } else {
        let row_count = results
            .iter()
            .map(|entry| entry.translations.len())
            .sum::<usize>();
        let start_row = if row_count > max_rows {
            row_count - max_rows
        } else {
            0
        };
        let mut i: usize = 0;

        for entry in results {
            if let Some(word) = entry.word {
                let add_date_str = if let Some(add_date) = entry.add_date {
                    format!("{}", add_date)
                } else {
                    "                ".to_string()
                };

                let add_counter_str = if let Some(add_counter) = entry.add_counter {
                    format!(" ({})", add_counter)
                } else {
                    "".to_string()
                };

                for translation in entry.translations {
                    i += 1;
                    if i > start_row {
                        println!(
                            "{} {}: {}{}",
                            add_date_str, word, translation, add_counter_str
                        );
                    }
                }
            }
        }
    }
    println!();
}

fn add_to_personal_db(db_vocabulary: &mut Db, db_personal: &mut Db, number: usize) {
    let predicates = vec![Predicate::new_equal_int("search_index", number as i32)];
    let row_ids = db_vocabulary.select_row_ids(&predicates, Some(1));
    for entry in find_row_ids_to_entries(db_vocabulary, &row_ids) {
        if let Some(word) = entry.word {
            //println!("Adding {}: {}", word, entry.translations.join("; "));
            add_word(db_personal, &word, &entry.translations, true, true);
        }
    }
}

/// Searches for a search term. Will try the following:
/// - Exact search
/// - Exact search in conjugations ("tiene" -> "tener")
/// - Exact search in translations
/// - Search with String::starts_with()
/// - Search with String::contains()
fn find_and_display(db: &mut Db, search_term: &str, max_results: usize) {
    println!("\nSearch term: {}", search_term);

    let rows_equal = find_row_ids(
        &db,
        "name",
        search_term,
        PredicateType::Equal,
        Some(max_results),
    );
    let rows_conjugation = find_row_ids(
        &db,
        "conjugation",
        search_term,
        PredicateType::Equal,
        Some(max_results),
    );

    let number_equal = rows_equal.len();
    let number_conjugations = rows_conjugation.len();
    if number_equal == 0 && number_conjugations > 0 {
        println!("\nMatch in conjugations:");
        add_numbers(db, &rows_conjugation, number_conjugations);
        present(&db, &rows_conjugation, number_conjugations == max_results);
    } else {
        let rows_starts_with_full = find_row_ids(
            &db,
            "name",
            search_term,
            PredicateType::StartsWith,
            Some(max_results),
        );
        let rows_starts_with = minus(&rows_starts_with_full, &rows_equal);
        let number_starts_with = rows_starts_with_full.len();

        if number_starts_with < max_results {
            let rows_contains_full = find_row_ids(
                &db,
                "name",
                search_term,
                PredicateType::Contains,
                Some(max_results),
            );
            let number_contains = rows_contains_full.len();

            if number_contains == 0 {
                let mut new_search_term = search_term.to_string();
                new_search_term.pop();
                if new_search_term.len() >= 3 {
                    println!(
                        "{} not found. Searching for {} instead.",
                        search_term, new_search_term
                    );
                    find_and_display(db, &new_search_term, max_results);
                    return;
                } else {
                    println!("\n{} not found.", search_term);
                }
            } else {
                let rows_contains = minus(&rows_contains_full, &rows_starts_with_full);
                if !rows_contains.is_empty() {
                    println!("\nFull matches:");
                    add_numbers(db, &rows_contains, number_starts_with);
                    present(&db, &rows_contains, number_contains == max_results);
                }
            }
        }

        if number_starts_with > 0 && !rows_starts_with.is_empty() {
            println!("\nStarting with:");
            add_numbers(db, &rows_starts_with, number_equal);
            present(&db, &rows_starts_with, number_starts_with == max_results);
        }

        if number_equal > 0 {
            println!("\nEquals:");
            add_numbers(db, &rows_equal, 0);
            present(&db, &rows_equal, number_equal == max_results);
        }
    }
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
        let predicates = vec![Predicate::new_any_string("value")];
        let entries = vec![String::from("value")];
        let row_ids = db.select_row_ids(&predicates, None);
        let words = row_ids.len();
        let result = db.entries_from_row_ids(&row_ids, entries);
        let translations = result.iter().map(|entry| entry.len()).sum::<usize>();
        println!("Saved {} words and {} translations.", words, translations);
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
        let filename = "resources/es-en/es-en-sample.txt";
        let (mut db, _) = load_dictionary(dbname, "dummy", filename, true);

        let values = find(&mut db, "coche", PredicateType::Equal);
        assert_eq!(values.len(), 1);
        assert_eq!(values[0].word, Some("coche".to_string()));
        assert_eq!(values[0].translations[0], "car");

        let values = find(&mut db, "coche", PredicateType::StartsWith);
        assert_eq!(values.len(), 4);
        assert_eq!(values[2].word, Some("coche eléctrico".to_string()));
        assert_eq!(values[2].translations[0], "electric car");

        let values = find(&mut db, "coche", PredicateType::Contains);
        assert_eq!(values.len(), 5);
        assert_eq!(values[4].word, Some("lavacoches".to_string()));
        assert_eq!(values[4].translations[0], "carwash");
    }

    #[test]
    fn conjugations() {
        let dbname = "test-sample";
        let filename = "resources/es-en/es-en-sample.txt";
        let conjugations_filename = "resources/irregular_verbs/irregular_verbs_sample.txt";
        let (mut db, _) = load_dictionary(dbname, "dummy", filename, true);
        load_irregular_verbs(&mut db, filename, conjugations_filename);

        // "cueces" is a conjugation of "cocer" which is RowId(8)
        let row_ids = find_row_ids(&db, "conjugation", "cueces", PredicateType::Equal, None);
        println!("row_ids: {:?}", row_ids);
        assert_eq!(row_ids, [RowId(8)]);
    }

    #[test]
    fn find_row_ids_to_entries() {
        let dbname = "test-sample";
        let filename = "resources/es-en/es-en-sample.txt";
        let conjugations_filename = "resources/irregular_verbs/irregular_verbs_sample.txt";
        let (mut db, _) = load_dictionary(dbname, "dummy", filename, true);
        load_irregular_verbs(&mut db, filename, conjugations_filename);
        let row_ids = find_row_ids(&db, "conjugation", "cueces", PredicateType::Equal, None);
        let entries = super::find_row_ids_to_entries(&db, &row_ids);
        println!("{:?}", entries);
        assert_eq!(entries[0].translations.len(), 2);
        assert_eq!(entries[0].conjugations.len(), 15);
    }
}
