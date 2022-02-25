# Team Plover bevy jam 1 game

## Local setup

What you need to get things running:
* A working [rust](https://www.rust-lang.org/) environment
* (optional but recommended) lld linker [to drastically improve compile
  times](https://bevyengine.org/learn/book/getting-started/setup/#enable-fast-compiles-optional)

## Building

* `cargo clippy` to check if it compiles
* `cargo run` to run the game

### Compiling release builds

```sh
cargo build --release --no-default-features

# The executable is the following file (we run strip to remove the debug signs,
# this reduces the size of the distributed executable):
strip target/release/jambevy
```

I setup the `dynamic` feature so that we can use bevy's dynamic linking feature
when developing. Dynamic linking is great to lower compile time, but it makes
it difficult to distribute the game (because it requires distributing several
binaries, and because it doesn't work really well on Windows) so it should be
disabled when building the release binary. To disable it, you just pass the
`--no-default-features` flag to cargo when building.


## Development guidelines

This is the wild west, **do what you think should be done**. Rust makes it
*insanely easy* to properly document and test your code, so you should consider
it more so than you are used to, because it won't eat your time as much as in
other programming languages.

### Error handling

Rust has a lot of features and helper "crates" (dependencies) to manage errors
gracefully, however in the first
iterative process, you should consider just using `.unwrap()`, we'll grep later
to fix those.

### Performance

Feel free to add `.clone()` everywhere, thanks to lifetime tracking, the
compiler is excellent at eliminating useless copies at compile time. We should
spend time optimizing only when performance is a problem. [Check out the rust
perf book for general
guidelines](https://nnethercote.github.io/perf-book/title-page.html)

### Compilation speed

For game making, quick iteration is paramount. Thankfully bevy is designed for
this! [check the bevy
doc](https://bevyengine.org/learn/book/getting-started/setup/#enable-fast-compiles-optional).
In my experience, the compile times go from 5 seconds to about 0.5 second.

### Commits

Before a commit, make sure to run the following:
```
cargo fmt
cargo clippy
cargo test
```

To install those tools, read on
* [clippy](https://github.com/rust-lang/rust-clippy) `rustup component add clippy`
* [rustfmt](https://github.com/rust-lang/rustfmt) `rustup component add rustfmt`

I personally always re-read the code two or three times before a commit, but
that's just the way I do things.

#### Commit messages

I recommend you use [this
convention](https://chris.beams.io/posts/git-commit/) (Subject line 50 chars
max, imperative mood, capitalized)

Meaningfull commit messages and self-contained commits are extremely useful for
long running projects. Since this is a 7 days project, it's not that important ;)

### Tooling

* `rust_analyzer` is a great tool, especially when discovering a new and complex
  API.
* `cargo doc --open` is your friend, it let you generate the docs for the game
  code itself **and all its dependencies**, you can then browse them locally.

## Architecture

The game is built on `Plugin`s. Each module will define the `Component`s,
`system`s and `Res`ources it uses. Expose publicly the ones that are to be used
by other modules and the `Plugin`s gets loaded in `main`.
