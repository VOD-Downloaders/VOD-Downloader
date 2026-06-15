# Contributing

Contributions are highly appreciated (especially to the [fronted](#fronted-html-css-js) and [documentation](#documentation-markdown)).  

## Backend (Rust, Docker)

To contribute to the backend follow these steps:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/my-change`)
3. Make your changes and ensure everything compiles (`cargo build` && `docker build .`)
4. Run tests (`cargo test`)
5. Run the linter (`cargo clippy`)
6. Format your code (`cargo +nightly fmt` from [`rustfmt`](https://github.com/rust-lang/rustfmt))
7. Open a pull request with a clear description of what you changed and why

## Fronted (HTML, CSS, JS)

Contributions to the WebUI are highly appreciated. Most of the current fronted is written with the help of AI. I would like a human rewrite at some point.
To contribute to the fronted follow these steps:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/my-change`)
3. Make your changes
4. Open a pull request with a clear description of what you changed and why

## Documentation (Markdown)

Contributions to the Documentation are highly appreciated, add your files to the [`doc/`](./doc) folder.
