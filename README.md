# Torrust Index

[![coverage_wf_b]][coverage_wf] [![deployment_wf_b]][deployment_wf] [![testing_wf_b]][testing_wf]

This repository serves the [Torrust Index](https://github.com/torrust/torrust-index) project, which implements the [Torrust Index Application Interface](https://github.com/torrust/torrust-index-api-lib).

We also provide the [Torrust Index Gui](https://github.com/torrust/torrust-index-gui) project, which is our reference web application that consumes the API provided here.

![Torrust Architecture](https://raw.githubusercontent.com/torrust/.github/main/img/torrust-architecture.webp)

## Key Features

* [X] Rest API
* [X] Categories and tags
* [X] Image proxy cache

## Getting Started

Requirements:

* Rust Stable `1.72`

You can follow the [documentation](https://docs.rs/torrust-index) to install and use Torrust Index in different ways, but if you want to give it a quick try, you can use the following commands:

```s
git clone https://github.com/torrust/torrust-index.git \
  && cd torrust-index \
  && cargo build --release
```

And then run `cargo run` twice. The first time to generate the `config.toml` file and the second time to run the index with the default configuration.

After running the index the API will be available at <http://localhost:3001>.

## Documentation

The technical documentation is available at [docs.rs](https://docs.rs/torrust-index).

## Contributing

This is an open-source community supported project.</br>
We welcome contributions from the community!

__How can you contribute?__

- Bug reports and feature requests.
- Code contributions. You can start by looking at the issues labeled "[good first issues]".
- Documentation improvements. Check the [documentation] and [API documentation] for typos, errors, or missing information.
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


[coverage_wf]: ../../actions/workflows/coverage.yaml
[coverage_wf_b]: ../../actions/workflows/coverage.yaml/badge.svg
[deployment_wf]: ../../actions/workflows/deployment.yaml
[deployment_wf_b]: ../../actions/workflows/deployment.yaml/badge.svg
[testing_wf]: ../../actions/workflows/testing.yaml
[testing_wf_b]: ../../actions/workflows/testing.yaml/badge.svg


[good first issues]: https://github.com/torrust/torrust-index/issues?q=is%3Aissue+is%3Aopen+label%3A%22good+first+issue%22
[documentation]: https://docs.rs/torrust-index/
[API documentation]: https://docs.rs/torrust-index/latest/torrust_index/servers/apis/v1
[discussions]: https://github.com/torrust/torrust-index/discussions

[AGPL_3_0]: ./docs/licenses/LICENSE-AGPL_3_0
[MIT_0]: ./docs/licenses/LICENSE-MIT_0
[FSF]: https://www.fsf.org/
