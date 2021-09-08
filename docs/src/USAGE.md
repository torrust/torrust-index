# Getting started
The easiest way is to get built binaries from [Releases](https://github.com/torrust/torrust/releases),
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
