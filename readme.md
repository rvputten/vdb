Vocabulary Trainer
==================
Motivation: Have fun

Ideas
=====
flat file to store vocabulary
  might implement basics of a database engine
    separate module for database engine
flexible number of columns
  general aspect of file storage
    could be used for all kinds of data
      might end up being a database editor instead of a vocabulary trainer, with extensions for vocabulary training
has several date columns
  create date
  history of "remembered" and "don't know" answers
    rerun after one day
    another rerun after 5 days
    another rerun after 30 days
automatically detect inflections
  pollito -> pollo

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
