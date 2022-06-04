# Torrust Index Backend

This repository serves as the backend for the [Torrust Index](https://github.com/torrust/torrust) project.

## Documentation
You can read the documentation [here](https://torrust.github.io/torrust-documentation/torrust-web-backend/about/).

## Installation
The easiest way is to get built binaries from [Releases](https://github.com/torrust/torrust-web-backend/releases),
but building from sources is also possible:

```bash
git clone https://github.com/torrust/torrust.git
cd torrust
cargo build --release
```

__Notice:__ Skip the first step if you've downloaded the binaries directly.

1. After building __Torrust__, navigate to the folder.
```bash
cd torrust/target
```

2. Create a file called `config.toml` with the following contents and change the [configuration](https://torrust.github.io/torrust-tracker/CONFIG.html) according to your liking.


3. And run __Torrust__:
```bash
./torrust
```

## Contributing
Please report any Torrust Index backend specific bugs you find to the issue tracker of this repository. Torrust Index frontend specific issues can be submitted [here](https://github.com/torrust/torrust-index-frontend). Universal issues with the Torrust Index can be submitted [here](https://github.com/torrust/torrust). Ideas and feature requests are welcome as well!
