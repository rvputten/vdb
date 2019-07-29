API draft
=========

The following are examples how interacting with the database might work in the future.

Example usage and comparision with SQL. Note that there are no table names so we use 'all' instead
SQL-like queries:

#. Update an existing row with an incrementing number

In SQL:

```sql
    update words
    set add_counter = coalesce(add_counter + 1, 1)
    where name = "coche"
    limit 1; -- we know we only want to update one row - not easy in standard sql
```

In vdb:

```rust
    let row_id = db.equals("name", "coche").next().unwrap();
    let counter = if let Some(counter) = row_id.db_int("add_counter") {
	counter + 1
    } else {
	1
    };
    row_id.upsert("add_counter", Db::db_int(counter));
```

#. Update existing column with the current date

In SQL:

```sql
    update words
    set add_date = current timestamp
    where name = "coche";
```

In vdb:

```rust
    db.equals("name", "coche")
	.next()
	.unwrap()
	.upsert("add_date", Data::now());
```

#. Add a word in Spanish with multiple translations in English

In SQL:

```sql
    insert into translations (name, translation)
    values
	("nube", "cloud"),
	("nube", "crowd of people");

    insert into words (name, add_date, add_count)
    values ("nube", current timestamp, 1);
```

In vdb:

```rust
    let row_id = db.add_row(vec![
	    Entry::new_string("name", "nube"),
	    Entry::new_date("add_date", Data::now()),
	    Entry::new_int("add_counter", 1),
    ]);
    let _row_id = db.add_row(vec![Entry::parent("word", row_id), Entry::new_string("translation", "cloud")]);
    let _row_id = db.add_row(vec![Entry::parent("word", row_id), Entry::new_string("translation", "crowd of people")]);
```

#. Search for "coche" and display translations

In SQL:

```sql
    select
	add_date || " " ||
	name || ": " ||
	translation
    from words
    join translations on words.name = translations.name
    where name = "coche";
```

In vdb:

```rust
    for word_id in db.equals("name", "coche") {
	for translation_id in word_id.children("word") {
	    println!("{} {}: {}",
		    word_id.db_date("add_date"),
		    word_id.db_string("name"),
		    translation_id.db_string("translation"),
	    );
	}
    }
```

#. Add employee table and a calendar for who is on standy duty and their backup

In SQL:

```sql
    insert into employee (emp_id, name)
    values
	(1, "Mueller"),
	(2, "Lambart");

    insert into duty (date, standby_id, backup_id)
    values (2017-12-25, 1, 2);
```

In vdb:

```rust
    let emp1_id = db.add_row(vec![Entry::new_int("emp_id", 1), Entry::new_string("name", "Mueller")]);
    let emp2_id = db.add_row(vec![Entry::new_int("emp_id", 2), Entry::new_string("name", "Lambart")]);
    let _duty_id= db.add_row(vec![
	    Entry::new_date("date", "2017-12-25"),
	    Entry::new_parent("standby", emp1_id),
	    Entry::new_parent("backup", emp2_id),
    ]);
```

#. Find the person who is on standby and their backup

In SQL:

```sql
    select
	duty.date || ": " ||
	standby.name || " " ||
       	backup.name
    from
	employee as standby,
	employee as backup,
	duty
    where
	duty.standby_id = standby.emp_id
	and duty.backup_id = backup.emp_id
	and duty.date = 2017-12-25;
```

In vdb:

```rust
    let duty_id = db.equals_date("date", Data::Date::From("2017-12-25")).next().unwrap();
    let standby_id = db.parent("standby_id", duty_id);
    let backup_id  = db.parent("backup_id" , duty_id);
    println!("{}: {} {}",
	    duty_id.db_date("date"),
	    standby_id.db_string("name"),
	    backup_id.db_string("name"),
    );
```

#. Find the number of employees

In SQL:

```sql
    select count(*) from employee;
```

In vdb:

```rust
    db.by_name("emp_id").count(*);
```

#. Query with multiple where clauses

In SQL:

```sql
    select
	add_date,
       	add_count,
       	word,
       	translation
    from
	word,
       	translation
    where
	word.name = translation.name
	and add_count > 1
	and add_date > 2017-01-01;
```

In vdb:

```rust
    for word_id in db
	.greater_than_i32("add_count", 1)
	.greater_than_date("add_date", "2017-01-01")
    {
	for translation_id in word_id.children("translation") {
	    println!("{} {}: {} ({})",
		    word_id.db_date("date"),
		    word_id.db_string("name"),
		    translation.db_string("translation"),
		    word_id.db_i32("add_count"),
	    );
	}
    }
```
