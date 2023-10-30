# `surveilr` Resource Surveillance

This Rust-based project is an extendable file system inspector for performing surveillance of machine resources.

```bash
$ cargo run -- --help                         # get CLI help
$ cargo run -- fs-walk --help                 # get CLI help for fs-walk subcommand
$ cargo run -- fs-walk                        # walk the current working directory (CWD)
$ cargo run -- fs-walk -r /other -r /other2   # walk some other director(ies)
$ cargo run -- fs-walk -i .git/               # walk CWD, ignore .git/ paths
$ cargo run -- fs-walk -i .git/ -i target/    # walk CWD, ignore .git/ and target/ paths
```

