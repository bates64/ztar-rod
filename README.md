**ztar rod** &emdash; paper mario modification tool
###### [discord server](https://discord.gg/88vy32w)

---

# what is this

ztar rod is a work-in-progress romhacking tool for [Paper Mario](https://wikipedia.org/wiki/Paper_Mario). It is currently able to dump map script bytecode to source-code and parse it back into an AST.

Tons of credit goes to cloverhax, who did most of the reverse-engineering and released _Star Rod_.
ZR exists as a more performant, featureful (soon, hopefully!) and less buggy alternative to it,
since Clover disappeared.

# usage

You will need:
- [Rust](https://rustup.rs/)
- A legally-sourced _Paper Mario (USA)_ ROM.

With `Paper Mario (U) [!].z64` in the working-directory:
```sh
$ cargo run
```

It'll eventually panic as some operations are not yet unimplemented, but it should populate the `mod/` directory.

# license

This software is licensed under the UNLICENSE; i.e. it has been released into the public domain. You can do whatever with it: redistribute, sell, modify, etc.
