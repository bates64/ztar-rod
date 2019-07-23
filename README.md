# what is this

ztar rod is a work-in-progress romhacking tool for [Paper Mario](https://wikipedia.org/wiki/Paper_Mario).

Tons of credit goes to cloverhax, who did most of the reverse-engineering and released _Star Rod_.
ZR exists as a more performant, featureful (soon!) and less buggy alternative to it, since Clover
disappeared.

# usage

You will need:
- [Rust](https://rustup.rs/) version `1.34.0` or higher
- A USA, PAL, or JP _Paper Mario_ ROM:
  * `Paper Mario (U) [!].z64`
  * `Paper Mario (Europe) (End,Fr,De,Es).z64`
  * `Mario Story (J) [!].z64`

With the rom file in your working-directory:
```sh
$ cargo run
```

# license

ztar rod is licensed under the [Apache License, Version 2.0](http://www.apache.org/licenses/LICENSE-2.0) or the [MIT license](http://opensource.org/licenses/MIT), at your option.
