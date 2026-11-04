# Arch packaging

`PKGBUILD` builds `cheatrs-bin`: it takes the statically linked binary straight from
the GitHub release and installs it with `LICENSE` and `NOTICE`. Nothing compiles, so
it installs in a second and does not depend on a Rust toolchain.

```bash
cd packaging
makepkg -si
```

## Why there is no source package

A source PKGBUILD that runs `cargo build` was written and tested first. It fails
under makepkg and only under makepkg:

```
rust-lld: error: undefined symbol: onig_region_resize
rust-lld: error: undefined symbol: onig_new
rust-lld: error: undefined symbol: OnigEncodingUTF8
```

`syntect` links `onig_sys`, which compiles the oniguruma C library. The same tarball
builds cleanly with `cargo build --release --locked`, with `--frozen`, and with
makepkg's `LDFLAGS` and `CFLAGS` exported by hand — but not inside `makepkg`, where
the archive ends up with its symbols unresolved. The rlib does contain them
(`nm` finds `onig_new`), so it is a linking-order problem between makepkg's
environment and rustc's static-archive bundling, not a missing dependency.

Two ways out, neither taken yet:

1. Build `syntect` with the pure-Rust `fancy-regex` backend instead of oniguruma.
   That removes the C dependency entirely and would also make static musl builds
   possible without `musl-gcc`.
2. Track down the exact link-order interaction with makepkg's environment.

Until one of them lands, `-bin` is the package that works, and a source build is
still available to anyone with `cargo install --path .` from the tarball.

## Submitting to the AUR

The AUR needs an account and an SSH key; the file here is ready to push as
`cheatrs-bin/PKGBUILD` with a generated `.SRCINFO`:

```bash
makepkg --printsrcinfo > .SRCINFO
```
