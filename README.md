# `surveilr` Resource Surveillance

This Rust-based project is an extendable file system inspector for performing surveillance of machine resources.

```bash
$ cargo run -- --help                         # get CLI help
$ cargo run -- fs-walk --help                 # get CLI help for fs-walk subcommand
$ cargo run -- fs-walk                        # walk the current working directory (CWD)
$ cargo run -- fs-walk -r /other              # walk some other directory
$ cargo run -- fs-walk -i .git/               # walk CWD, ignore .git/ paths
$ cargo run -- fs-walk -i .git/ -i target/    # walk CWD, ignore .git/ and target/ paths
```

## TODO

- [ ] Use regex::RegexSet to use looping of RegEx
- [ ] Use [Deno Core](https://crates.io/crates/deno_core) for configuration language
  * See https://news.ycombinator.com/item?id=35816224  (ChatGPT can help too): 
    * https://deno.com/blog/roll-your-own-javascript-runtime
    * https://deno.com/blog/roll-your-own-javascript-runtime-pt2
    * https://deno.com/blog/roll-your-own-javascript-runtime-pt3
  * Another option (not preferred but use if Deno does not work) is or https://crates.io/crates/nickel-lang-core
- [ ] Use https://crates.io/crates/vfs for abstracting file system(s)
  - [ ] https://crates.io/crates/vfs-zip
  - [ ] https://crates.io/crates/vfs-clgit
  - [ ] https://crates.io/crates/vfs-https
- [ ] Use https://crates.io/crates/directories for loading configs, etc. from known locations
