extern crate chrono;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

//use chrono::{DateTime, Duration, Utc};
use chrono::{Local, NaiveDateTime};
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::Path;

/// A basic database system to store key/value pairs with few dependencies.
///
/// # Examples
///
/// ```
/// use vdb::{Db, Entry, Predicate};
/// let mut db = Db::new("test-db");
/// let row_1 = db.add(vec![
///         Entry::new_string("word", "cocina"),
///         Entry::new_string("translation", "cuisine"),
///         Entry::new_string("translation", "kitchen"),
/// ]);
/// let row_2 = db.add(vec![
///         Entry::new_string("word", "coche"),
///         Entry::new_string("translation", "car"),
/// ]);
///
/// // Load and save
/// db.save();
/// let mut new_db = Db::load("test-db").unwrap();
/// let row_ids = new_db.find_all_row_ids();
/// assert_eq!(row_ids.len(), 2);
///
/// // Find rows
/// let row_ids = db.select_row_ids(&vec![Predicate::new_equal_string("word", "coche")], None);
/// assert_eq!(row_ids, [row_2]);
/// let entries = db.entries_from_row_ids(&row_ids, &["translation"]);
/// assert_eq!(entries[0][0], Entry::new_string("translation", "car"));
///
/// // Delete
/// let coche = db.find_first_row_id_by_value("word", &Db::db_string("coche"));
/// assert_eq!(coche, Some(row_2));
/// db.delete_rows(&[row_1, row_2]);
/// let no_coche = db.find_first_row_id_by_value("word", &Db::db_string("coche"));
/// assert_eq!(no_coche, None);
/// ```

/// Data types currently implemented in the database
#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub enum Data {
    DbString(String),
    DbInt(i32),
    DbDateTime(NaiveDateTime),
}

impl fmt::Display for Data {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let printable = match self {
            Data::DbDateTime(date_time) => date_time.format("%Y-%m-%d %H:%M").to_string(),
            Data::DbInt(number) => format!("{}", number),
            Data::DbString(string) => string.clone(),
        };
        write!(f, "{}", printable)
    }
}

impl Data {
    /// Tests if the data starts with the given string
    fn starts_with(&self, data: &Data) -> bool {
        if let (Data::DbString(left), Data::DbString(right)) = (self, data) {
            left.starts_with(right)
        } else {
            false
        }
    }

    /// Tests if the data contains the given string
    fn contains(&self, data: &Data) -> bool {
        if let (Data::DbString(left), Data::DbString(right)) = (self, data) {
            left.contains(right)
        } else {
            false
        }
    }

    /// Returns new DbDateTime with current time as timestamp
    pub fn now() -> Data {
        Data::DbDateTime(Local::now().naive_local())
    }

    pub fn date(&self) -> Option<String> {
        if let Data::DbDateTime(d) = self {
            Some(d.format("%Y-%m-%d").to_string())
        } else {
            None
        }
    }
}

/// The Row Identifier is used to reference each data set and is used by many methods where the
/// actual data is not used directly.
#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug, Copy, PartialOrd, Ord)]
pub struct RowId(pub usize);

/// Each RowId has many entries. Comparable to column name+data in relational databases.
#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct Entry {
    pub name: String,
    pub value: Data,
}

impl Entry {
    /// Shortcut for creating a new `Entry` with a `DbString`
    pub fn new_string(name: &str, value: &str) -> Entry {
        Entry {
            name: String::from(name),
            value: Db::db_string(value),
        }
    }

    /// # Examples
    ///
    /// ```
    /// use vdb::{Entry,Predicate};
    /// let a = Entry::new_string("mundo", "world");
    ///
    /// assert_eq!(a.compare(&Predicate::new_equal_string("mundo", "world")), true);
    /// assert_eq!(a.compare(&Predicate::new_contains("mundo", "orl")), true);
    /// assert_eq!(a.compare(&Predicate::new_equal_string("mundo", "World")), false);
    /// ```
    pub fn compare(&self, predicate: &Predicate) -> bool {
        match &predicate.predicate_type {
            PredicateType::Any => predicate.entry.name == self.name,
            PredicateType::Equal => {
                predicate.entry.name == self.name && predicate.entry.value == self.value
            }
            PredicateType::StartsWith => {
                self.name == predicate.entry.name && self.value.starts_with(&predicate.entry.value)
            }
            PredicateType::Contains => {
                self.name == predicate.entry.name && self.value.contains(&predicate.entry.value)
            }
        }
    }

    /// Return first `Entry` in a given list that matches `name`
    pub fn get_first_by_name(entries: &[Entry], name: &str) -> Option<Entry> {
        for entry in entries {
            if entry.name == name {
                return Some(entry.clone());
            }
        }
        None
    }
}

#[derive(PartialEq, Debug)]
pub enum PredicateType {
    Equal,
    StartsWith,
    Contains,
    Any,
}

/// Used to compare database entries, e. g. in queries (fn find_*)
///
/// # Examples
///
/// ```
/// use vdb::{Entry,Predicate};
/// let a = Entry::new_string("mundo", "world");
///
/// assert_eq!(a.compare(&Predicate::new_equal_string("mundo", "world")), true);
/// assert_eq!(a.compare(&Predicate::new_starts_with("mundo", "worl")), true);
/// assert_eq!(a.compare(&Predicate::new_contains("mundo", "orl")), true);
/// assert_eq!(a.compare(&Predicate::new_equal_string("mundo", "planet")), false);
/// ```
#[derive(Debug)]
pub struct Predicate {
    pub predicate_type: PredicateType,
    pub entry: Entry,
}

impl Predicate {
    /// Shortcut for creating a new `Predicate` that tests for equality with a `DbInt`
    pub fn new_equal_int(name: &str, value: i32) -> Predicate {
        Predicate {
            predicate_type: PredicateType::Equal,
            entry: Entry {
                name: String::from(name),
                value: Db::db_int(value),
            },
        }
    }

    /// Shortcut for creating a new `Predicate` that searches database for `DbString`s equal to
    /// `value`
    pub fn new_any_string(name: &str) -> Predicate {
        Predicate {
            predicate_type: PredicateType::Any,
            entry: Entry {
                name: String::from(name),
                value: Db::db_string(""),
            },
        }
    }

    /// Shortcut for creating a new `Predicate` that searches database for `DbString`s equal to
    /// `value`
    pub fn new_equal_string(name: &str, value: &str) -> Predicate {
        Predicate {
            predicate_type: PredicateType::Equal,
            entry: Entry {
                name: String::from(name),
                value: Db::db_string(value),
            },
        }
    }

    /// Shortcut for creating a new `Predicate` that searches database for `DbString`s starting
    /// with `value`
    pub fn new_starts_with(name: &str, value: &str) -> Predicate {
        Predicate {
            predicate_type: PredicateType::StartsWith,
            entry: Entry {
                name: String::from(name),
                value: Db::db_string(value),
            },
        }
    }
    /// Shortcut for creating a new `Predicate` that searches database for `DbString`s that contain
    /// `value`
    pub fn new_contains(name: &str, value: &str) -> Predicate {
        Predicate {
            predicate_type: PredicateType::Contains,
            entry: Entry {
                name: String::from(name),
                value: Db::db_string(value),
            },
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
struct Row {
    pub row_id: RowId,
    pub entry: Entry,
}

/// Container for the database. Usually only one is used per application.
///
/// # Examples
///
/// ```
/// use vdb::{Db, Entry};
/// let mut db = Db::new("test-db");
/// let _row_id = db.add(vec![Entry::new_string("mundo", "world")]);
/// ```
#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct Db {
    full_filename: String,
    row_max: RowId,
    rows: Vec<Row>,
}

impl Db {
    /// Create new database in memory. The file is not created until `save()` is called.
    pub fn new(filename: &str) -> Db {
        Db {
            full_filename: Db::build_filename(filename),
            row_max: RowId(0),
            rows: vec![],
        }
    }

    /// Load a database file from the filesystem under the subdirectory `save/`.
    ///
    /// # Errors
    ///
    /// May return errors from external modules while opening the file or parsing the contents.
    pub fn load(filename: &str) -> Result<Db, Box<Error>> {
        let full_filename = Db::build_filename(filename);
        let mut file = File::open(full_filename)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let result = serde_json::from_str(&contents)?;
        Ok(result)
    }

    /// Save database under the subdirectory `save/` with the same name it was `open`ed or `create`d
    /// with. The subdirectory `save/` must exist.
    pub fn save(&self) -> Result<(), Box<Error>> {
        let path = Path::new(&self.full_filename);
        let mut file = File::create(&path)?;
        let serialized = serde_json::to_string_pretty(self)?;
        file.write_all(serialized.as_bytes())?;
        Ok(())
    }

    pub fn db_string(v: &str) -> Data {
        Data::DbString(String::from(v))
    }

    pub fn db_int(v: i32) -> Data {
        Data::DbInt(v)
    }

    /// Parse `&str` into a `DbDateTime`. The format string is `%Y-%m-%d %H:%M:%S`.
    pub fn db_datetime(v: &str) -> Result<Data, Box<Error>> {
        let fmt = "%Y-%m-%d %H:%M:%S";
        let r = NaiveDateTime::parse_from_str(v, fmt)?;
        Ok(Data::DbDateTime(r))
    }

    /// Add a new row with multiple entries.
    pub fn add(&mut self, entries: Vec<Entry>) -> RowId {
        let id = self.next();
        for e in entries {
            self.rows.push(Row {
                row_id: id,
                entry: e,
            });
        }
        id
    }

    /// Add a single entry to an existing row. An existing entry with the same name is overwritten.
    pub fn add_or_update_entry(&mut self, row_id: RowId, entry: Entry) {
        // check if entry exists
        if let Some(ref mut db_entry) = self.get_first_entry_mut(row_id, &entry.name) {
            db_entry.value = entry.value;
        } else {
            self.rows.push(Row { row_id, entry });
        }
    }

    /// Add a single entry to an existing row. Does not check if entry exists.
    pub fn add_entry(&mut self, row_id: RowId, entry: Entry) {
        self.rows.push(Row { row_id, entry });
    }

    /// Delete rows in the database
    ///
    /// # Examples
    ///
    /// ```
    /// use vdb::{Db, Entry};
    /// let mut db = Db::new("test-db");
    /// let row_1 = db.add(vec![
    ///         Entry::new_string("word", "cocina"),
    ///         Entry::new_string("translation", "cuisine"),
    ///         Entry::new_string("translation", "kitchen"),
    /// ]);
    /// let row_2 = db.add(vec![
    ///         Entry::new_string("word", "coche"),
    ///         Entry::new_string("translation", "car"),
    /// ]);
    /// let coche = db.find_first_row_id_by_value("word", &Db::db_string("coche"));
    /// assert_eq!(coche, Some(row_2));
    /// db.delete_rows(&[row_1, row_2]);
    /// let no_coche = db.find_first_row_id_by_value("word", &Db::db_string("coche"));
    /// assert_eq!(no_coche, None);
    /// ```
    pub fn delete_rows(&mut self, row_ids: &[RowId]) {
        self.rows.retain(|row| !row_ids.contains(&row.row_id));
    }

    /// Delete all entries with this name in the whole database.
    pub fn delete_entry_all(&mut self, name: &str) {
        self.rows.retain(|row| row.entry.name != name);
    }

    /// Return row_ids of entries where an entry with name "name" exists.
    pub fn find_by_name(&self, name: &str) -> Vec<RowId> {
        let mut row_ids = self
            .rows
            .iter()
            .filter(|row| row.entry.name == name)
            .map(|row| row.row_id)
            .collect::<Vec<RowId>>();
        row_ids.sort();
        row_ids.dedup();
        row_ids
    }

    /// Return row_ids of entries that are exactly "value". For partial string matches, use
    /// Predicates.
    pub fn find_by_value(&self, name: &str, value: &Data) -> Vec<RowId> {
        let mut row_ids = self
            .rows
            .iter()
            .filter(|row| row.entry.name == name && &row.entry.value == value)
            .map(|row| row.row_id)
            .collect::<Vec<RowId>>();
        row_ids.sort();
        row_ids.dedup();
        row_ids
    }

    /// Return reference to first entry found in a given row.
    pub fn find_first_row_id_by_name(&self, name: &str) -> Option<RowId> {
        for row in &self.rows {
            if row.entry.name == name {
                return Some(row.row_id);
            }
        }
        None
    }

    /// Return reference to first entry found in a given row.
    pub fn find_first_row_id_by_value(&self, name: &str, value: &Data) -> Option<RowId> {
        for row in &self.rows {
            if row.entry.name == name && &row.entry.value == value {
                return Some(row.row_id);
            }
        }
        None
    }

    /// Return reference to first entry found in a given row.
    pub fn get_first_entry(&self, row_id: RowId, name: &str) -> Option<&Entry> {
        for row in &self.rows {
            if row.row_id == row_id && row.entry.name == name {
                return Some(&row.entry);
            }
        }
        None
    }

    /// Return mutable reference to an entry in a given row.
    pub fn get_first_entry_mut(&mut self, row_id: RowId, name: &str) -> Option<&mut Entry> {
        for row in &mut self.rows {
            if row.row_id == row_id && row.entry.name == name {
                return Some(&mut row.entry);
            }
        }
        None
    }

    /// Returns all rows if no predicates are given.
    /// The first predicate is evaluated first and should have high selectivity, i. e. evaluate to a
    /// small number of rows, to improve execution time. The number of results can be limited with
    /// `Some(max_results)`
    ///
    /// # Examples
    ///
    /// ```
    /// // Like SQL "select name, value from testdb where name='coche' limit 15"
    /// use vdb::{Data, Db, Entry, Predicate, RowId};
    /// let mut db = Db::new("test-db");
    /// let _id = db.add(vec![
    ///     Entry {
    ///         name: String::from("set"),
    ///         value: Db::db_string("es-en"),
    ///     },
    ///     Entry {
    ///         name: String::from("name"),
    ///         value: Db::db_string("coche"),
    ///     },
    ///     Entry {
    ///         name: String::from("value"),
    ///         value: Db::db_string("car"),
    ///     },
    /// ]);
    /// let predicates = vec![Predicate::new_equal_string("name", "coche")];
    /// let row_ids = db.select_row_ids(&predicates, Some(15));
    /// assert_eq!(row_ids, [RowId(1)]);
    /// assert_eq!(db.entries_from_row_ids(&row_ids, &["name", "value"])[0][0], Entry::new_string("name", "coche"));
    /// ```
    /// See also select()
    pub fn select_row_ids(
        &self,
        predicates: &[Predicate],
        max_results: Option<usize>,
    ) -> Vec<RowId> {
        let max_results = if let Some(max_results) = max_results {
            max_results
        } else {
            self.rows.len()
        };

        if predicates.is_empty() {
            let mut row_ids = self
                .rows
                .iter()
                .take(max_results)
                .map(|row| row.row_id)
                .collect::<Vec<RowId>>();
            row_ids.sort();
            row_ids.dedup();
            row_ids
        } else {
            let predicate0 = &predicates[0];
            let mut row_ids = self
                .rows
                .iter()
                .filter(|row| row.entry.compare(predicate0))
                .map(|row| row.row_id)
                .collect::<Vec<RowId>>();

            for predicate in &predicates[1..] {
                let new_row_ids = row_ids
                    .iter()
                    .filter(|&row_id| self.match_row(*row_id, predicate))
                    .cloned()
                    .collect::<Vec<RowId>>();
                row_ids = new_row_ids;
            }
            if max_results < row_ids.len() {
                let _ = row_ids.drain(max_results..).collect::<Vec<RowId>>();
            }
            row_ids.sort();
            row_ids.dedup();
            row_ids
        }
    }

    /// Returns all rows in the database
    pub fn find_all_row_ids(&self) -> Vec<RowId> {
        let mut row_ids = self
            .rows
            .iter()
            .map(|row| row.row_id)
            .collect::<Vec<RowId>>();
        row_ids.sort();
        row_ids.dedup();
        row_ids
    }

    #[cfg(test)]
    pub fn select(&self, predicates: &[Predicate], entries: &[&str]) -> Vec<Vec<Entry>> {
        let row_ids = self.select_row_ids(predicates, None);
        self.entries_from_row_ids(&row_ids, entries)
    }

    /// Returns entries for given row_ids.
    pub fn entries_from_row_ids(&self, row_ids: &[RowId], names: &[&str]) -> Vec<Vec<Entry>> {
        let names = names.iter().map(|s| s.to_string()).collect::<Vec<String>>();
        let mut result: Vec<Vec<Entry>> = vec![];
        for row_id in row_ids {
            let row = self
                .rows
                .iter()
                .filter(|row| row.row_id == *row_id && names.contains(&(&row.entry.name)))
                .map(|row| row.entry.clone())
                .collect::<Vec<Entry>>();

            let mut ordered: Vec<Entry> = vec![];
            for name in &names {
                for entry in row.iter().filter(|entry| &entry.name == name) {
                    ordered.push(entry.clone());
                }
            }

            result.push(ordered);
        }
        result
    }

    #[cfg(test)]
    fn has(&self, row_id: RowId, predicate: &Entry) -> bool {
        if let Some(_has) = self.rows.iter().find(|&row| {
            row.row_id == row_id
                && row.entry.name == predicate.name
                && row.entry.value == predicate.value
        }) {
            true
        } else {
            false
        }
    }

    /// Check if a predicate is true for a given row_id.
    fn match_row(&self, row_id: RowId, predicate: &Predicate) -> bool {
        if let Some(_has) = self
            .rows
            .iter()
            .find(|&row| row.row_id == row_id && row.entry.compare(predicate))
        {
            true
        } else {
            false
        }
    }

    fn next(&mut self) -> RowId {
        self.row_max.0 += 1;
        self.row_max
    }

    fn build_filename(name: &str) -> String {
        format!("save/{}", name)
    }

    #[cfg(test)]
    pub fn debug_rows(&self, row_ids: &[RowId]) -> Vec<Vec<Entry>> {
        let mut result: Vec<Vec<Entry>> = vec![];
        for row_id in row_ids {
            result.push(
                self.rows
                    .iter()
                    .filter(|row| row.row_id == *row_id)
                    .map(|row| row.entry.clone())
                    .collect::<Vec<Entry>>(),
            );
        }
        result
    }
}

mod tests {
    #[cfg(test)]
    use super::{Data, Db, Entry, Predicate, RowId};
    #[cfg(test)]
    use chrono::NaiveDateTime;

    #[test]
    fn match_row() {
        let db = new_db_with_entries("testdb");

        let p1 = Predicate::new_equal_string("name", "coche");
        let p2 = Predicate::new_starts_with("name", "co");
        let p3 = Predicate::new_contains("name", "och");

        println!("{:?}", db.debug_rows(&vec![RowId(2)]));
        println!("{:?}", db.debug_rows(&vec![RowId(1)]));

        assert_eq!(db.match_row(RowId(2), &p1), true);
        assert_eq!(db.match_row(RowId(2), &p2), true);
        assert_eq!(db.match_row(RowId(2), &p3), true);

        assert_eq!(db.match_row(RowId(1), &p1), false);
        assert_eq!(db.match_row(RowId(1), &p2), false);
        assert_eq!(db.match_row(RowId(1), &p3), false);
    }

    #[test]
    fn starts_with_contains() {
        let s1 = Db::db_string("hello");
        let s2 = Db::db_string("hello world");
        let s3 = Db::db_string("o wor");
        assert!(s2.starts_with(&s1));
        assert_eq!(s1.starts_with(&s2), false);
        assert!(s2.contains(&s3));
        assert_eq!(s3.contains(&s2), false);
    }

    #[test]
    fn compare() {
        let p1 = Predicate::new_equal_string("set", "es-en");
        let p2 = Predicate::new_starts_with("set", "es");
        let p3 = Predicate::new_contains("set", "s-e");

        let e1 = Entry {
            name: String::from("set"),
            value: Db::db_string("es-en"),
        };

        let e2 = Entry {
            name: String::from("set"),
            value: Db::db_string("en-es"),
        };

        assert!(e1.compare(&p1));
        assert_eq!(e2.compare(&p1), false);

        assert!(e1.compare(&p2));
        assert_eq!(e2.compare(&p2), false);

        assert!(e1.compare(&p3));
        assert_eq!(e2.compare(&p3), false);
    }

    #[cfg(test)]
    fn new_db_with_entries(name: &str) -> Db {
        let mut db = Db::new(name);
        let _id = db.add(vec![
            Entry {
                name: String::from("set"),
                value: Db::db_string("es-en"),
            },
            Entry {
                name: String::from("name"),
                value: Db::db_string("disfrutar"),
            },
            Entry {
                name: String::from("value"),
                value: Db::db_string("to enjoy"),
            },
        ]);
        let _id = db.add(vec![
            Entry {
                name: String::from("set"),
                value: Db::db_string("es-en"),
            },
            Entry {
                name: String::from("name"),
                value: Db::db_string("coche"),
            },
            Entry {
                name: String::from("value"),
                value: Db::db_string("car"),
            },
        ]);
        db
    }

    #[cfg(test)]
    fn check_single_entries(db: &Db) {
        assert_eq!(db.rows.len(), 6);
        assert_eq!(db.rows[0].row_id, RowId(1));
        assert_eq!(db.rows[0].entry.name, "set");
        assert_eq!(db.rows[0].entry.value, Db::db_string("es-en"));

        assert_eq!(db.rows[5].row_id, RowId(2));
        assert_eq!(db.rows[5].entry.name, "value");
        assert_eq!(db.rows[5].entry.value, Db::db_string("car"));
    }

    #[test]
    fn has() {
        let name = "testdb";
        let db = new_db_with_entries(name);
        assert!(db.has(
            RowId(1),
            &Entry {
                name: String::from("set"),
                value: Db::db_string("es-en")
            }
        ));
        assert_eq!(
            db.has(
                RowId(1),
                &Entry {
                    name: String::from("set"),
                    value: Db::db_string("does not exist")
                }
            ),
            false
        );
    }

    #[test]
    fn select_row_ids() {
        let name = "testdb";
        let db = new_db_with_entries(name);

        let predicates1 = vec![Predicate::new_equal_string("set", "es-en")];
        let predicates2 = vec![
            Predicate::new_equal_string("set", "es-en"),
            Predicate::new_equal_string("name", "disfrutar"),
        ];

        let row_ids = db.select_row_ids(&predicates1, None);
        assert_eq!(row_ids, vec![RowId(1), RowId(2)]);

        let row_ids = db.select_row_ids(&predicates2, None);
        assert_eq!(row_ids, vec![RowId(1)]);
    }

    #[test]
    fn select() {
        let name = "testdb";
        let db = new_db_with_entries(name);

        let predicates = vec![Predicate::new_equal_string("set", "es-en")];

        let result1 = vec![
            vec![Entry {
                name: String::from("name"),
                value: Db::db_string("disfrutar"),
            }],
            vec![Entry {
                name: String::from("name"),
                value: Db::db_string("coche"),
            }],
        ];

        let result2 = vec![
            vec![
                Entry {
                    name: String::from("name"),
                    value: Db::db_string("disfrutar"),
                },
                Entry {
                    name: String::from("value"),
                    value: Db::db_string("to enjoy"),
                },
            ],
            vec![
                Entry {
                    name: String::from("name"),
                    value: Db::db_string("coche"),
                },
                Entry {
                    name: String::from("value"),
                    value: Db::db_string("car"),
                },
            ],
        ];

        let result = db.select(&predicates, &vec!["name"]);
        assert_eq!(result, result1);

        let result = db.select(&predicates, &vec!["name", "value"]);
        assert_eq!(result, result2);
    }

    #[test]
    fn load_and_save() {
        let name = "testdb";
        let db = new_db_with_entries(name);
        let _result = db.save();
        let db = Db::load(name).unwrap();
        check_single_entries(&db);
    }

    #[test]
    fn add() {
        let db = new_db_with_entries("testdb");
        check_single_entries(&db);
    }

    #[test]
    fn data_types() {
        let t = "Test";
        assert_eq!(Data::DbString(String::from(t)), Db::db_string(t));
        let t = 42;
        assert_eq!(Data::DbInt(t), Db::db_int(t));
        let fmt = "%Y-%m-%d %H:%M:%S";
        let t = "2013-11-22 12:00:00";
        let dt = NaiveDateTime::parse_from_str(t, fmt).unwrap();
        assert_eq!(Data::DbDateTime(dt), Db::db_datetime(t).unwrap());
    }

    #[test]
    fn get_first_entry_mut() {
        let mut db = new_db_with_entries("testdb");

        if let Some(ref mut entry) = db.get_first_entry_mut(RowId(2), "name") {
            entry.name = String::from("replaced_name");
            println!("{:?}", entry);
        }
        println!("{:?}", db.debug_rows(&vec![RowId(2)]));
        for (i, row) in db.rows.iter().enumerate() {
            println!("{} {:?}", i, row);
        }
        assert_eq!(db.rows[4].entry.name, "replaced_name");
    }

    #[test]
    fn add_entry_add() {
        let mut db = new_db_with_entries("testdb");

        println!("{:?}", db.debug_rows(&vec![RowId(2)]));
        db.add_or_update_entry(
            RowId(2),
            Entry {
                name: String::from("new entry"),
                value: Db::db_string("new entry content"),
            },
        );
        println!("{:?}", db.debug_rows(&vec![RowId(2)]));
        for (i, row) in db.rows.iter().enumerate() {
            println!("{} {:?}", i, row);
        }
        assert_eq!(db.rows[6].entry.name, "new entry");
        assert_eq!(db.rows[6].entry.value, Db::db_string("new entry content"));
    }

    #[test]
    fn add_entry_update() {
        let mut db = new_db_with_entries("testdb");

        println!("{:?}", db.debug_rows(&vec![RowId(2)]));
        db.add_or_update_entry(
            RowId(2),
            Entry {
                name: String::from("new entry"),
                value: Db::db_string("new entry content"),
            },
        );
        db.add_or_update_entry(
            RowId(2),
            Entry {
                name: String::from("new entry"),
                value: Db::db_string("new entry content updated"),
            },
        );
        println!("{:?}", db.debug_rows(&vec![RowId(2)]));
        for (i, row) in db.rows.iter().enumerate() {
            println!("{} {:?}", i, row);
        }
        assert_eq!(db.rows[6].entry.name, "new entry");
        assert_eq!(
            db.rows[6].entry.value,
            Db::db_string("new entry content updated")
        );
    }

    #[test]
    fn delete_entry_all() {
        let mut db = new_db_with_entries("testdb");

        let mut add_row = |n| {
            println!("{:?}", db.debug_rows(&vec![RowId(n)]));
            db.add_entry(
                RowId(n),
                Entry {
                    name: String::from("new entry"),
                    value: Db::db_string(&format!("new col for row {}", n)),
                },
            );
        };
        add_row(1);
        add_row(2);
        add_row(3);
        for (i, row) in db.rows.iter().enumerate() {
            println!("{} {:?}", i, row);
        }
        assert_eq!(db.rows[6].entry.name, "new entry");

        println!("Deleting entries...");
        db.delete_entry_all("new entry");
        for (i, row) in db.rows.iter().enumerate() {
            println!("{} {:?}", i, row);
        }
        assert_eq!(db.rows.len(), 6);
        assert_eq!(db.rows[0].entry.name, "set");
        assert_eq!(db.rows[4].entry.name, "name");
    }

    #[test]
    fn find_all_row_ids() {
        let db = new_db_with_entries("testdb");
        assert_eq!(db.find_all_row_ids(), vec![RowId(1), RowId(2)]);
    }
}
