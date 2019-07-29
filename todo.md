To do
=====

model relationships
-------------------
extend enum Data to include RowId
```
enum Data {
    DbRowId(RowId),
    ..
}
```

speed up searches
-----------------
store everything in hashmaps
```
HashMap<
    name,
    HashMap<name, RowId, value>
    HashMap<value, RowId, name>
    HashMap<RowId, name, value>
>
```


use builder pattern to query database
-------------------------------------
```
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
```

import(filename, field1, field2, ..., fieldn) - load from a file

cleanup api
-----------
```
    add_int()
    add_or_update_int()
    update_int()
    add_string()
    find_first_int()
    find_ints()
    etc.
```
