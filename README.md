# db_connections

Traverse a directory and extract database connection information from ASP.NET and classic ASP files, (possibly) creating two files as output: a CSV file for connections and a text file for errors, which will be placed in the directory this program is run from.

If the directory to be traversed isn't provided explicitly (e.g. `cargo run -- some/path`), it will be set to the current working directory.

## Tests

Unit tests can be run with `cargo test`, and rely in part on the files in the test_files directory.
