# `BonsaiDb` + `Gooey` Example: Counter

This example shows how to create a [`BonsaiDb`](https://github.com/khonsulabs/bonsaidb/) server with a custom API and use [`Gooey`](https://github.com/khonsulabs/gooey/) to create a user interface to call that API. The example is simple: a button that increments a counter.

[![Example Screen Capture](https://khonsulabs.github.io/gooey/bonsaidb-counter-example.webp)](https://khonsulabs.github.io/gooey/bonsaidb-counter-example.webp)

There are three crates in this example:

- `bonsaidb-counter-shared`: Contains the API definition that the server exposes.
- `bonsaidb-counter-server`: A `BonsaiDb` server.
- `bonsaidb-counter-client`: The `Gooey` client application.

## Running the server

`cargo run --package bonsaidb-counter-server`

## Running the client (native)

`cargo run --package bonsaidb-counter-client`

## Trying the browser client

These steps rely on these tools:

### Required Tools

- [`wasm-bindgen-cli`](https://rustwasm.github.io/wasm-bindgen/reference/cli.html): Used to generate the glue code that [`wasm-bindgen`](https://github.com/rustwasm/wasm-bindgen) needs to interoperate with Javascript.

  ```cargo install wasm-bindgen-cli```

  **Note**: Gooey does not enforce a specific wasm-bindgen version, but the wasm-bindgen version used in the project must match the version of the command line tool. In general, this crate should always use the current version, and the above command installs the current version. If you get an error about version mismatch, look in the `Cargo.lock` file to determine what verison of `wasm-bindgen` is being used.

- [`miniserve`](https://github.com/svenstaro/miniserve): Used to serve the HTML, WebAssembly, and Javascript files in [`./browser/`](./browser).

  ```cargo install miniserve```

### Steps

These steps are written to work if your current working directory is `{repository}/integrated-examples/bonsaidb/counter`.

1. Build the client:

   `cargo xtask build-browser-example bonsaidb-counter-client`

2. Launch the HTTP server:

   `miniserve browser/`

3. Navigate to the website: `http://localhost:8080/index.html`
