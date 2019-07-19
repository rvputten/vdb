vdb - a database system implemented in rust
===========================================

A database system that takes ideas from Entity Component Systems and relational databases.

State of the project
--------------------
*   no dependencies except for serde and chrono
*   loads and saves
*   add/update key/value pairs
*   search for keys/values

Planned
-------
*   delete
*   incremental updates to the save files
    *   not calling Vdb::save() will not lose data
*   model dependencies between stored keys
*   speed improvements
    *	use of hashmaps
    *	reverse lookup (search for key or RowId)
    *	separate store for each key
    *	indexes
    *	partitions
*   use of enums instead of &str for keys
*   use builder pattern to query database
*   bigger & smaller comparisons (a > b)

Further in the future
---------------------
*   allow access from multiple threads
*   client-server architecture
*   binary data storage (with or without serde)
*   tooling for schema upgrades
