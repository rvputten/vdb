Short-term ideas for database
=============================
make own crate and publish on crates.io
example code in examples/
use enum instead of &str for column names
    enum Columns {
	Word,
	Translation
    }
    db.add([db::entry(Word, Db::db_string("P100D")), db::entry(Translation, Db::db_string("ludicrous"))]);
store hierarchies (see above)
    extend enum Data to include RowId
	enum Data {
	    DbRowId(RowId),
	    ..
	}
speed up searches
    store everything in hashmaps
	HashMap<
	    name,
	    HashMap<ByRowId>
	    HashMap<ByEntryName>
	>
    reverse lookup
	needs two hashmaps (name, row_id) per entry for quick lookup

Long-term ideas for database
============================
immediately store all changes
    upon load, first load the old database, then the changes
migration of databases
    no solution yet, is it possible with serde?
    possibly needs an external program to load & save different versions
	like diesel
diesel integration

Actions
=======
create new entry
query+take entry from dictionary
query entries
modify entry
delete entry
start drill
  all
  new words only
  old words only

Table layout
============
    set		    - name of the set this answer belongs to
    source	    - the word in the source language
    translation	    - multiple fields possible; translations
    create_date	    - date the entry was created
    answer_date/[yes|no] - date an answer was given; multiple fields possible

Database capabilities
=====================
entity component system
  not a relational database
all values are always encoded
operations:
    add([
	    ["set", "spa-eng"],
	    ["source", "galleta"],
	    ["translation", ["biscuit", "cookie"]],
	    ["create_date", "2019-07-04-17:14:02"],
	    ["answer_date", [["2019-07-04-17:15:15", "no"], ["2019-07-04-17:17:28", "no"], ["2019-07-04-18:01:40", "yes"]],
	])

    query() could be implemented as an iterator; is <> filter, fields <> map
    query()
	.is("set", "spa-eng")
	.begins("source", "gall")
	.fields(["source", "translation"])
    query/filter operations:
	.is(attribute, value)
	.begins(attribute, value)
	.fields()
	.contains(attribute, value)
	.distinct()
	.sort()
	.uniq()
	.reverse()
	.take()

    import(filename, field1, field2, ..., fieldn) - load from a file
