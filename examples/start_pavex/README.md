# Leptos Pavex Starter

# Getting started

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install)
- [Pavex]
- [Leptos](https://leptos.dev)
- [`cargo-px`]
- [`cargo-leptos`](https://github.com/leptos-rs/cargo-leptos)
- _(Optional but recommended)_ [`cargo-hakari`](https://docs.rs/cargo-hakari/0.9.29/cargo_hakari/index.html#installation)

## Useful commands

This website is built using the [Leptos] full stack web framework on top of the [Pavex] backend server. [Pavex] relies on code generation.  
You need to use the `cargo px` command instead of `cargo` to run the server: it ensures that the
`server_sdk` crate is correctly regenerated when the application blueprint changes.

[`cargo-px`] is a wrapper around `cargo` that will automatically regenerate the
server SDK when needed. 
Check out its [documentation](https://github.com/LukeMathWalker/cargo-px)
for more details.

While ['cargo-leptos'] has been modified to handle Pavex, it is a work in progress. See below for details

### Build

```bash
# Build pavex and Leptos.
cargo leptos build
```

### Run

Leptos requires a bunch of environment vars to be set to function properly. Unforunately cargo-leptos does not yet run `cargo px run`
for you automatically. To show the list of env vars that need to be set, you can run `cargo leptos -vv`, which will show you an output like this

```bash
Cargo envs: LEPTOS_OUTPUT_NAME=start_pavex LEPTOS_SITE_ROOT=target/site LEPTOS_SITE_PKG_DIR=pkg LEPTOS_SITE_ADDR=127.0.0.1:3000 LEPTOS_RELOAD_PORT=3001 LEPTOS_LIB_DIR=frontend LEPTOS_BIN_DIR=server LEPTOS_JS_MINIFY=false LEPTOS_HASH_FILES=false
```

These can then be added to your shell however you'd like, for example by copy pasting before the command on Linux/Mac.

```bash
# You can also use `cargo px r`, if you prefer.
LEPTOS_OUTPUT_NAME=start_pavex LEPTOS_SITE_ROOT=target/site LEPTOS_SITE_PKG_DIR=pkg LEPTOS_SITE_ADDR=127.0.0.1:3000 LEPTOS_RELOAD_PORT=3001 LEPTOS_LIB_DIR=frontend LEPTOS_BIN_DIR=server LEPTOS_JS_MINIFY=false LEPTOS_HASH_FILES=false cargo px run
```

NOTE: While these won't change unless you change the cargo-leptos settings, please don't copy paste the env vars from here. Some of them may
be different on your machine

The command above will launch Leptos and Pavex, which will start listening on
port `8000`. The API will use the `dev` profile. Check out [CONFIGURATION.md] for more details.

### Test

```bash
# You can also use `cargo px t`, if you prefer.
cargo px test
```

## Configuration

The configuration system used by this application is detailed in [CONFIGURATION.md].

## `workspace_hack`

The project includes a "utility" crate named `workspace_hack`. It is used
to speed up project builds by forcing eager feature unification.  
To understand how it works, check out [`cargo-hakari`'s documentation](https://docs.rs/cargo-hakari/0.9.29/cargo_hakari/about/index.html).  
To keep `workspace_hack` up to date, install `cargo-hakari` and run:

```bash
cargo hakari generate && cargo hakari manage-deps -y
```

[Pavex]: https://pavex.dev
[Leptos]: https://leptos.dev
[`cargo-px`]: https://lukemathwalker.github.io/cargo-px/
[`cargo-leptos`]: https://github.com/leptos-rs/cargo-leptos/
[CONFIGURATION.md]: CONFIGURATION.md
