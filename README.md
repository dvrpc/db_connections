# db_connections

Traverse directories and extract database connection information from ASP.NET (.aspx and .config) and classic ASP (.asp) files, (possibly) creating two files as output: a CSV file for connections and a CSV file for errors, which will be placed in the directory this program is run from.

The directories to traverse (recursively) can be specified as arguments to the program - e.g. `cargo run -- some/path/ some/other/path`. If no command line arguments are provided, the current directory will be traversed.

A log of the program's activity is created in the directory the program is run from.

## Tests

Unit tests can be run with `cargo test`, and rely in part on the files in the test_files directory.
