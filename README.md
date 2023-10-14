# Torrust Index

[![container_wf_b]][container_wf] [![coverage_wf_b]][coverage_wf] [![deployment_wf_b]][deployment_wf] [![testing_wf_b]][testing_wf]

__Torrust Index__, is a [BitTorrent][bittorrent] Index (a service where a group of people can maintain a set of torrents and their associated metadata), written in [Rust Language][rust] and [axum] (a modern web application framework). ___This index aims to be respectful to established standards, (both [formal][BEP 00] and [otherwise][torrent_source_felid]).___

> This is a [Torrust][torrust] project and is in active development. It is community supported as well as sponsored by [Nautilus Cyberneering][nautilus].

- _We have a [container guide][containers.md] for those who wish to get started with __Docker__ or __Podman___

## Key Features

- [x] High Quality and Modern Rust Codebase.
- [x] [Rest API][documentation] documentation generated from code comments.
- [x] [Comprehensive Suit][coverage] of Unit and Functional Tests.
- [x] Good Performance in Busy Conditions.
- [x] Native `IPv4` and `IPv6` support.
- [x] Support for either `SQLite3` or `MySQL` databases.
- [x] Categories and Tags
- [x] Image Proxy

## Getting Started

### Container Version

The Torrust Index is [deployed to DockerHub][dockerhub], you can run a demo immediately with the following commands:

#### Docker:

```sh
docker run -it torrust/index:develop
```
> Please read our [container guide][containers.md] for more information.

#### Podman:

```sh
podman run -it torrust/index:develop
```
> Please read our [container guide][containers.md] for more information.

### Development Version

Requirements:

* Rust Stable `1.72`

You can follow the [documentation][documentation] to install and use Torrust Index in different ways, but if you want to give it a quick try, you can use the following commands:

```s
git clone https://github.com/torrust/torrust-index.git \
  && cd torrust-index \
  && cargo build --release
```

And then run `cargo run` twice. The first time to generate the `config.toml` file and the second time to run the index with the default configuration.

After running the index the API will be available at <http://localhost:3001>.

## Documentation

The technical documentation is available at [docs.rs][documentation].

## Contributing

This is an open-source community supported project.</br>
We welcome contributions from the community!

__How can you contribute?__

- Bug reports and feature requests.
- Code contributions. You can start by looking at the issues labeled "[good first issues]".
- Documentation improvements. Check the [documentation] and [API documentation][api] for typos, errors, or missing information.
- Participation in the community. You can help by answering questions in the [discussions].

## License

**Copyright (c) 2023 The Torrust Developers.**

This program is free software: you can redistribute it and/or modify it under the terms of the [GNU Affero General Public License][AGPL_3_0] as published by the [Free Software Foundation][FSF], version 3.

This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the [GNU Affero General Public License][AGPL_3_0] for more details.

You should have received a copy of the *GNU Affero General Public License* along with this program. If not, see <https://www.gnu.org/licenses/>.

Some files include explicit copyright notices and/or license notices.

### Legacy Exception

For prosperity, versions of Torrust Index that are older than five years are automatically granted the [MIT-0][MIT_0] license in addition to the existing [AGPL-3.0-only][AGPL_3_0] license.

## Contributions

The copyright of the Torrust Index is retained by the respective authors.

**Contributors agree:**
- That all their contributions be granted a license(s) **compatible** with the [Torrust Index License](#License).
- That all contributors signal **clearly** and **explicitly** any other compilable licenses if they are not: *[AGPL-3.0-only with the legacy MIT-0 exception](#License)*.

**The Torrust-Index project has no copyright assignment agreement.**

## Acknowledgments

This project was a joint effort by [Nautilus Cyberneering GmbH](https://nautilus-cyberneering.de/), [Dutch Bits](https://dutchbits.nl) and collaborators.  Thank you to you all!


[container_wf]: ../../actions/workflows/container.yaml
[container_wf_b]: ../../actions/workflows/container.yaml/badge.svg
[coverage_wf]: ../../actions/workflows/coverage.yaml
[coverage_wf_b]: ../../actions/workflows/coverage.yaml/badge.svg
[deployment_wf]: ../../actions/workflows/deployment.yaml
[deployment_wf_b]: ../../actions/workflows/deployment.yaml/badge.svg
[testing_wf]: ../../actions/workflows/testing.yaml
[testing_wf_b]: ../../actions/workflows/testing.yaml/badge.svg

[bittorrent]: http://bittorrent.org/
[rust]: https://www.rust-lang.org/
[axum]: https://github.com/tokio-rs/axum
[coverage]: https://app.codecov.io/gh/torrust/torrust-index
[torrust]: https://torrust.com/

[dockerhub]: https://hub.docker.com/r/torrust/index/tags
[torrent_source_felid]: https://github.com/qbittorrent/qBittorrent/discussions/19406

[containers.md]: ./docs/containers.md

[documentation]: https://docs.rs/torrust-index-backend/latest/torrust_index_backend/
[api]: https://docs.rs/torrust-index-backend/latest/torrust_index_backend/web/api/v1/

[good first issues]: https://github.com/torrust/torrust-index/issues?q=is%3Aissue+is%3Aopen
[discussions]: https://github.com/torrust/torrust-index/discussions

[AGPL_3_0]: ./docs/licenses/LICENSE-AGPL_3_0
[MIT_0]: ./docs/licenses/LICENSE-MIT_0
[FSF]: https://www.fsf.org/
