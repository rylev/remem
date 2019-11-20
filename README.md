<h1 align="center">remem</h1>
<div align="center">
 <strong>
   Thread-safe object reuse
 </strong>
</div>

<br />

<div align="center">
  <!-- Crates version -->
  <a href="https://crates.io/crates/remem">
    <img src="https://img.shields.io/crates/v/remem.svg?style=flat-square"
    alt="Crates.io version" />
  </a>
  <!-- Downloads -->
  <a href="https://crates.io/crates/remem">
    <img src="https://img.shields.io/crates/d/remem.svg?style=flat-square"
      alt="Download" />
  </a>
  <!-- docs.rs docs -->
  <a href="https://docs.rs/remem">
    <img src="https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square"
      alt="docs.rs docs" />
  </a>
</div>

<div align="center">
  <h3>
    <a href="https://docs.rs/remem">
      API Docs
    </a>
    <span> | </span>
    <a href="https://github.com/rylev/remem/blob/master/.github/CONTRIBUTING.md">
      Contributing
    </a>
  </h3>
</div>

## Installation

With [cargo add](https://github.com/killercup/cargo-edit) installed run:

```sh
$ cargo add remem
```

## Safety
This crate uses two instances of `unsafe` to implement a lock-free Treiber
stack. This implementation was copied from the reference implementation in
`crossbeam`.

## Contributing
Want to join us? Check out our ["Contributing" guide][contributing] and take a
look at some of these issues:

- [Issues labeled "good first issue"][good-first-issue]
- [Issues labeled "help wanted"][help-wanted]

[contributing]: https://github.com/rylev/remem/blob/master.github/CONTRIBUTING.md
[good-first-issue]: https://github.com/rylev/remem/labels/good%20first%20issue
[help-wanted]: https://github.com/rylev/remem/labels/help%20wanted

## License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br/>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
</sub>
