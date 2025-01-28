
## Setup

* To quicken compile time (at least the linking part) install lld using `brew install lld`
* To get code coverage we use llvm-cov
```shell
rustup component add llvm-tools-preview
cargo install cargo-llvm-cov
```
* Add linting via clippy
```shell
rustup component add clippy
cargo clippy

## Fail on warnings
cargo clippy -- -D warnings
```
* add Formatting
```shell
rustup component add rustfmt
cargo fmt

## Fail on warnings
cargo fmt -- --check
```

* To deal with cves use cargo-audit
```shell
cargo install cargo-audit
cargo audit
```