//use chrono::{DateTime, Duration, Utc};
use chrono::{NaiveDateTime, ParseError};
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::{error, fmt};

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub enum Data {
    DbString(String),
    DbInt(i32),
    DbDateTime(NaiveDateTime),
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct RowId(usize);

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct Pair {
    name: String,
    value: Data,
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct Row {
    row_id: RowId,
    entry: Pair,
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct Db {
    full_filename: String,
    row_max: RowId,
    rows: Vec<Row>,
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
    pub fn add(&mut self, name: &str, value: Data) -> RowId {
        let id = self.next();
        self.rows.push(Row {
            row_id: id.clone(),
            entry: Pair {
                name: String::from(name),
                value,
            },
        });
        id
    }

    fn next(&mut self) -> RowId {
        self.row_max.0 += 1;
        self.row_max.clone()
    }
    fn build_filename(name: &str) -> String {
        format!("save/{}", name)
    }
}

mod test {
    use super::{Data, Db, RowId};
    use chrono::NaiveDateTime;

    fn new_db_with_entry(name: &str) -> Db {
        let mut db = Db::new(name);
        let value = Db::db_string("es-en");
        let id = db.add("set", value.clone());
        db
    }

    fn check_single_entry(db: &Db) {
        let value = Db::db_string("es-en");
        assert_eq!(db.rows.len(), 1);
        assert_eq!(db.rows[0].row_id, RowId(1));
        assert_eq!(db.rows[0].entry.name, "set");
        assert_eq!(db.rows[0].entry.value, value);
    }

    #[test]
    fn load_and_save() {
        let name = "testdb";
        let db = new_db_with_entry(name);
        db.save();
        let db = Db::load(name).unwrap();
        check_single_entry(&db);
    }

    #[test]
    fn add() {
        let db = new_db_with_entry("testdb");
        check_single_entry(&db);
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