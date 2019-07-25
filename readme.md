vdb - a database system implemented in rust
===========================================

A basic database system that takes ideas from Entity Component Systems and relational databases.

Run example with
```
cargo run --example notebook
```

State of the project
--------------------
*   no dependencies except for serde and chrono
*   loads and saves
*   add/update/delete key/value pairs
*   search for keys/values

Planned
-------
*   incremental updates to the save files
    *   not calling Vdb::save() will not lose data
*   model relationships between stored keys, like foreign keys
*   speed improvements
    *	separate store for each key
    *	indexes
    *	partitions
*   use of enums instead of &str for keys
*   use builder pattern to query database
*   bigger & smaller comparisons (a > b)
*   create examples

Further in the future
---------------------
*   allow access from multiple threads
*   client-server architecture
*   binary data storage (with or without serde)
*   tooling for schema upgrades
*   diesel integration
