# fuckmiit

## What's this

I don't know.

## Usage

```toml
[dependencies]
fuckmiit = { git = "https://github.com/pysio2007/fuckmiit" }
```

```rust
fn main() -> fuckmiit::Result<()> {
    // Raw queryDetailByServiceIdAndDomainId JSON body; `true` prints debug steps to stderr.
    let body = fuckmiit::query_domain("example.com", false)?;
    println!("{}", String::from_utf8_lossy(&body));
    Ok(())
}
```

or

```rust
let detail = fuckmiit::query_domain_detail("example.com", false)?;
println!("{}", detail.service_licence);
```

Low-level HTTP client: `fuckmiit::api::Client`.

## Development

Toolchain (cargo, rustc, clippy, rustfmt, rust-analyzer) is declared in `flake.nix`:

```sh
nix develop
cargo test
cargo run --example example
```

## Thank

- [Mxmilu666/fuckmiit](https://github.com/Mxmilu666/fuckmiit), the original Go implementation this Rust fork is ported from
- Daiyangcheng's dress img
- ChatGPT

## License
WTFPL License
