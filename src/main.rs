extern crate chrono;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

mod db;

use db::Db;

fn main() {
    let _db = Db::load("default");
    println!("Hello, world!");
}
