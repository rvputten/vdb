Vocabulary Trainer
==================
Motivation: Have fun

Source for vocabulary
=====================
https://github.com/mananoreboton/en-es-en-Dic.git

Source for irregular verbs
==========================
https://github.com/voldmar/conjugation.git

Ideas for database
==================
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
