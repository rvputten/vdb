extern crate vdb;

use std::io;
use std::io::Write;

use vdb::{Db, Entry};

fn list_entries(db: &mut Db) {
    let row_ids = db.find_row_ids_by_name("title");
    let entries = db.entries_from_row_ids(&row_ids, &["title", "text"]);
    if entries.is_empty() {
        println!();
        println!("No entries.");
    } else {
        for entry in &entries {
            if entry.len() >= 2 {
                println!("{}: {}", entry[0].value, entry[1].value);
            }
        }
    }
}

fn new_entry(db: &mut Db) {
    println!("Enter title:");
    print!("> ");
    io::stdout().flush().unwrap();

    let mut input = "".to_string();
    let title = {
        let _bytes_read = io::stdin().read_line(&mut input).unwrap();
        input.trim()
    };
    if !title.is_empty() {
        println!("Enter text:");
        print!("> ");
        io::stdout().flush().unwrap();
        let mut input = "".to_string();
        let _bytes_read = io::stdin().read_line(&mut input).unwrap();
        let text = input.trim();
        db.add_row(vec![
            Entry::new_string("title", title),
            Entry::new_string("text", text),
        ]);
    } else {
        println!("Abort.");
    }
}

fn delete_entry(db: &mut Db) {
    println!("Enter title to delete:");
    print!("> ");
    io::stdout().flush().unwrap();

    let mut input = "".to_string();
    let title = {
        let _bytes_read = io::stdin().read_line(&mut input).unwrap();
        input.trim()
    };
    if !title.is_empty() {
        let row_ids = db.find_row_ids_by_value("title", &Db::db_string(title));
        db.delete_rows(&row_ids);
    } else {
        println!("Abort.");
    }
}

fn print_menu() {
    println!();
    println!("Main menu");
    println!("---------");
    println!("l) list entries");
    println!("e) enter new entry");
    println!("d) delete entry");
    println!("q) save & quit");

    print!("> ");
    io::stdout().flush().unwrap();
}

fn main_loop(db: &mut Db) {
    let mut input = "".to_string();
    print_menu();
    while let Ok(_bytes_read) = io::stdin().read_line(&mut input) {
        let trimmed = input.trim();
        match trimmed {
            "l" => list_entries(db),
            "e" => new_entry(db),
            "d" => delete_entry(db),
            "" | "q" => {
                let _ = db.save();
                break;
            }
            _ => (),
        }
        print_menu();
        input.clear();
    }
}

fn main() {
    let db_name = "notebook";
    let mut db = if let Ok(db) = Db::load(db_name) {
        db
    } else {
        Db::new(db_name)
    };
    main_loop(&mut db);
}
