# Torrust Index Backend

[![Testing](https://github.com/torrust/torrust-index-backend/actions/workflows/testing.yaml/badge.svg)](https://github.com/torrust/torrust-index-backend/actions/workflows/testing.yaml) [![Publish crate](https://github.com/torrust/torrust-index-backend/actions/workflows/publish_crate.yml/badge.svg)](https://github.com/torrust/torrust-index-backend/actions/workflows/publish_crate.yml) [![Publish Docker Image](https://github.com/torrust/torrust-index-backend/actions/workflows/publish_docker_image.yml/badge.svg)](https://github.com/torrust/torrust-index-backend/actions/workflows/publish_docker_image.yml) [![Publish Github Release](https://github.com/torrust/torrust-index-backend/actions/workflows/release.yml/badge.svg)](https://github.com/torrust/torrust-index-backend/actions/workflows/release.yml) [![Test Docker build](https://github.com/torrust/torrust-index-backend/actions/workflows/test_docker.yml/badge.svg)](https://github.com/torrust/torrust-index-backend/actions/workflows/test_docker.yml)

This repository serves as the backend for the [Torrust Index](https://github.com/torrust/torrust-index) project, which implements the [Torrust Index Application Interface](https://github.com/torrust/torrust-index-api-lib).

We also provide the [Torrust Index Frontend](https://github.com/torrust/torrust-index-frontend) project, which is our reference web application that consumes the API provided here.

![Torrust Architecture](https://raw.githubusercontent.com/torrust/.github/main/img/torrust-architecture.webp)

## Key Features

* [X] Rest API
* [X] Categories and tags
* [X] Image proxy cache

## Getting Started

Requirements:

* Rust Stable `1.68`

You can follow the [documentation](https://docs.rs/torrust-index-backend) to install and use Torrust Index Backend in different ways, but if you want to give it a quick try, you can use the following commands:

```s
git clone https://github.com/torrust/torrust-index-backend.git \
  && cd torrust-index-backend \
  && cargo build --release
```

And then run `cargo run` twice. The first time to generate the `config.toml` file and the second time to run the backend with the default configuration.

After running the tracker the API will be available at <http://localhost:3001>.

## Documentation

The technical documentation is available at [docs.rs](https://docs.rs/torrust-index-backend).

## Contributing

We welcome contributions from the community!

How can you contribute?

* Bug reports and feature requests.
* Code contributions. You can start by looking at the issues labeled ["good first issues"](https://github.com/torrust/torrust-index-backend/issues?q=is%3Aissue+is%3Aopen+label%3A%22good+first+issue%22).
* Documentation improvements. Check the [documentation](torrust-index-backend) for typos, errors, or missing information.
* Participation in the community. You can help by answering questions in the [discussions](https://github.com/torrust/torrust-index-backend/discussions).

## License

The project is licensed under a dual license. See [COPYRIGHT](./COPYRIGHT).

## Acknowledgments

This project was a joint effort by [Nautilus Cyberneering GmbH](https://nautilus-cyberneering.de/), [Dutch Bits](https://dutchbits.nl) and collaborators.  Thank you to you all!
