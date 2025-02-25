# Coding conventions

We format the code using `rustfmt`. Run `cargo fmt` prior to committing the code.
Run `scripts/clippy.sh` to check the code for common mistakes with [Clippy].

[Clippy]: https://doc.rust-lang.org/clippy/

## SQL

Multi-line SQL statements should be formatted using string literals,
for example
```
    sql.execute(
        "CREATE TABLE messages (
id INTEGER PRIMARY KEY AUTOINCREMENT,
text TEXT DEFAULT '' NOT NULL -- message text
) STRICT",
    )
    .await?;
```

Do not use macros like [`concat!`](https://doc.rust-lang.org/std/macro.concat.html)
or [`indoc!](https://docs.rs/indoc).
Do not escape newlines like this:
```
    sql.execute(
        "CREATE TABLE messages ( \
id INTEGER PRIMARY KEY AUTOINCREMENT, \
text TEXT DEFAULT '' NOT NULL \
) STRICT",
    )
    .await?;
```
Escaping newlines
is prone to errors like this if space before backslash is missing:
```
"SELECT foo\
 FROM bar"
```
Literal above results in `SELECT fooFROM bar` string.
This style also does not allow using `--` comments.

---

Declare new SQL tables with [`STRICT`](https://sqlite.org/stricttables.html) keyword
to make SQLite check column types.

Declare primary keys with [`AUTOINCREMENT`](https://www.sqlite.org/autoinc.html) keyword.
This avoids reuse of the row IDs and can avoid dangerous bugs
like forwarding wrong message because the message was deleted
and another message took its row ID.

Declare all new columns as `NOT NULL`
and set the `DEFAULT` value if it is optional so the column can be skipped in `INSERT` statements.
Dealing with `NULL` values both in SQL and in Rust is tricky and we try to avoid it.
If column is already declared without `NOT NULL`, use `IFNULL` function to provide default value when selecting it.
Use `HAVING COUNT(*) > 0` clause
to [prevent aggregate functions such as `MIN` and `MAX` from returning `NULL`](https://stackoverflow.com/questions/66527856/aggregate-functions-max-etc-return-null-instead-of-no-rows).

Don't delete unused columns too early, but maybe after several months/releases, unused columns are
still used by older versions, so deleting them breaks downgrading the core or importing a backup in
an older version. Also don't change the column type, consider adding a new column with another name
instead. Finally, never change column semantics, this is especially dangerous because the `STRICT`
keyword doesn't help here.

## Errors

Delta Chat core mostly uses [`anyhow`](https://docs.rs/anyhow/) errors.
When using [`Context`](https://docs.rs/anyhow/latest/anyhow/trait.Context.html),
capitalize it but do not add a full stop as the contexts will be separated by `:`.
For example:
```
.with_context(|| format!("Unable to trash message {msg_id}"))
```

All errors should be handled in one of these ways:
- With `if let Err() =` (incl. logging them into `warn!()`/`err!()`).
- With `.log_err().ok()`.
- Bubbled up with `?`.

`backtrace` feature is enabled for `anyhow` crate
and `debug = 1` option is set in the test profile.
This allows to run `RUST_BACKTRACE=1 cargo test`
and get a backtrace with line numbers in resultified tests
which return `anyhow::Result`.

## Logging

For logging, use `info!`, `warn!` and `error!` macros.
Log messages should be capitalized and have a full stop in the end. For example:
```
info!(context, "Ignoring addition of {added_addr:?} to {chat_id}.");
```

Format anyhow errors with `{:#}` to print all the contexts like this:
```
error!(context, "Failed to set selfavatar timestamp: {err:#}.");
```
