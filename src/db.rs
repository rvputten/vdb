//use chrono::{DateTime, Duration, Utc};
use chrono::NaiveDateTime;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::Path;

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub enum Data {
    DbString(String),
    DbInt(i32),
    DbDateTime(NaiveDateTime),
}

impl Data {
    fn starts_with(&self, data: &Data) -> bool {
        if let Data::DbString(left) = self {
            if let Data::DbString(right) = data {
                return left.starts_with(right);
            }
        }
        false
    }
    fn contains(&self, data: &Data) -> bool {
        if let Data::DbString(left) = self {
            if let Data::DbString(right) = data {
                return left.contains(right);
            }
        }
        false
    }
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug, Copy)]
pub struct RowId(usize);

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct Entry {
    pub name: String,
    pub value: Data,
}

#[derive(PartialEq, Debug)]
pub enum PredicateType {
    Equal,
    StartsWith,
    Contains,
}

#[derive(Debug)]
pub struct Predicate {
    predicate_type: PredicateType,
    entry: Entry,
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct Row {
    pub row_id: RowId,
    pub entry: Entry,
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct Db {
    full_filename: String,
    row_max: RowId,
    pub rows: Vec<Row>,
}

impl Db {
    pub fn new(filename: &str) -> Db {
        Db {
            full_filename: Db::build_filename(filename),
            row_max: RowId(0),
            rows: vec![],
        }
    }
    pub fn load(filename: &str) -> Result<Db, Box<Error>> {
        let full_filename = Db::build_filename(filename);
        let mut file = File::open(full_filename)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let result = serde_json::from_str(&contents)?;
        Ok(result)
    }

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
    pub fn db_datetime(v: &str) -> Result<Data, Box<Error>> {
        let fmt = "%Y-%m-%d %H:%M:%S";
        let r = NaiveDateTime::parse_from_str(v, fmt)?;
        Ok(Data::DbDateTime(r))
    }

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

    fn select_row_ids(&self, predicates: &[Entry]) -> Vec<RowId> {
        if predicates.is_empty() {
            self.rows
                .iter()
                .map(|row| row.row_id)
                .collect::<Vec<RowId>>()
        } else {
            let predicate0 = &predicates[0];
            let mut row_ids = self
                .rows
                .iter()
                .filter(|row| {
                    row.entry.name == predicate0.name && row.entry.value == predicate0.value
                })
                .map(|row| row.row_id)
                .collect::<Vec<RowId>>();

            for predicate in &predicates[1..] {
                let new_row_ids = row_ids
                    .iter()
                    .filter(|&row_id| self.has(*row_id, predicate))
                    .cloned()
                    .collect::<Vec<RowId>>();
                row_ids = new_row_ids;
            }
            row_ids
        }
    }

    // The current implementation has run time of O(n2), so predicates[0] must have high selectivity.
    // For predicates[1..], low selectivity is ok.
    pub fn select(&self, predicates: &[Entry], columns: Vec<String>) -> Vec<Vec<Entry>> {
        let mut result: Vec<Vec<Entry>> = vec![];
        let row_ids = self.select_row_ids(predicates);
        for row_id in &row_ids {
            result.push(
                self.rows
                    .iter()
                    .filter(|row| row.row_id == *row_id && columns.contains(&(&row.entry.name)))
                    .map(|row| row.entry.clone())
                    .collect::<Vec<Entry>>(),
            );
        }
        result
    }

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

    fn next(&mut self) -> RowId {
        self.row_max.0 += 1;
        self.row_max
    }

    fn build_filename(name: &str) -> String {
        format!("save/{}", name)
    }

    fn compare(predicate: &Predicate, entry: &Entry) -> bool {
        let result = match &predicate.predicate_type {
            PredicateType::Equal => {
                predicate.entry.name == entry.name && predicate.entry.value == entry.value
            }
            PredicateType::StartsWith => {
                entry.name == predicate.entry.name
                    && entry.value.starts_with(&predicate.entry.value)
            }
            PredicateType::Contains => {
                entry.name == predicate.entry.name && entry.value.contains(&predicate.entry.value)
            }
            _ => panic!("Not implemented"),
        };
        result
    }
}

mod test {
    use super::{Data, Db, Entry, Predicate, PredicateType, RowId};
    use chrono::NaiveDateTime;

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
        let p1 = Predicate {
            predicate_type: PredicateType::Equal,
            entry: Entry {
                name: String::from("set"),
                value: Db::db_string("es-en"),
            },
        };

        let p2 = Predicate {
            predicate_type: PredicateType::StartsWith,
            entry: Entry {
                name: String::from("set"),
                value: Db::db_string("es"),
            },
        };

        let p3 = Predicate {
            predicate_type: PredicateType::Contains,
            entry: Entry {
                name: String::from("set"),
                value: Db::db_string("s-e"),
            },
        };

        let e1 = Entry {
            name: String::from("set"),
            value: Db::db_string("es-en"),
        };

        let e2 = Entry {
            name: String::from("set"),
            value: Db::db_string("en-es"),
        };

        assert!(Db::compare(&p1, &e1));
        assert_eq!(Db::compare(&p1, &e2), false);

        assert!(Db::compare(&p2, &e1));
        assert_eq!(Db::compare(&p2, &e2), false);

        assert!(Db::compare(&p3, &e1));
        assert_eq!(Db::compare(&p3, &e2), false);
    }

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

        let predicates1 = vec![Entry {
            name: String::from("set"),
            value: Db::db_string("es-en"),
        }];

        let predicates2 = vec![
            Entry {
                name: String::from("set"),
                value: Db::db_string("es-en"),
            },
            Entry {
                name: String::from("name"),
                value: Db::db_string("disfrutar"),
            },
        ];

        let row_ids = db.select_row_ids(&predicates1);
        assert_eq!(row_ids, vec![RowId(1), RowId(2)]);

        let row_ids = db.select_row_ids(&predicates2);
        assert_eq!(row_ids, vec![RowId(1)]);
    }

    #[test]
    fn select() {
        let name = "testdb";
        let db = new_db_with_entries(name);

        let predicates = vec![Entry {
            name: String::from("set"),
            value: Db::db_string("es-en"),
        }];

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

        let result = db.select(&predicates, vec![String::from("name")]);
        assert_eq!(result, result1);

        let result = db.select(
            &predicates,
            vec![String::from("name"), String::from("value")],
        );
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
}
