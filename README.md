freq
====

Counts number of `[a-zA-Z]+` words in given file or standard input and outputs it to a file or standard output.

Usage
-----

### Run

```bash
$> cargo run --release -- pg.txt out.txt
```

Run with echo:

```bash
$> echo "this is a test message with a duplicated words: 'message, test, this'" | cargo run --release -- -
    Finished release [optimized] target(s) in 0.03s
     Running `target/release/freq -`
2 a
2 message
2 test
2 this
1 duplicated
1 is
1 with
```

### Build

```bash
$> cargo build --release
    Finished release [optimized] target(s) in 0.06s
$> ./target/release/freq pg.txt out.txt
```
