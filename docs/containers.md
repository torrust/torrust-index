# Containers (Docker or Podman)

## Demo environment

It is simple to setup the index with the default
configuration and run it using the pre-built public docker image:

With Docker:

```sh
docker run -it torrust/index:latest
```

or with Podman:

```sh
podman run -it torrust/index:latest
```

## Requirements

- Tested with recent versions of Docker or Podman.

## Volumes

The [Containerfile](../Containerfile) defines three volumes:

```Dockerfile
VOLUME ["/var/lib/torrust/index","/var/log/torrust/index","/etc/torrust/index"]
```

When instancing the container image with the `docker run` or `podman run` command, we map these volumes to the local storage:

```s
./storage/index/lib -> /var/lib/torrust/index
./storage/index/log -> /var/log/torrust/index
./storage/index/etc -> /etc/torrust/index
```

> NOTE: You can adjust this mapping for your preference, however this mapping is the default in our guides and scripts.

### Pre-Create Host-Mapped Folders

Please run this command where you wish to run the container:

```sh
mkdir -p ./storage/index/lib/ ./storage/index/log/ ./storage/index/etc/
```

### Matching Ownership ID's of Host Storage and Container Volumes

It is important that the `torrust` user has the same uid `$(id -u)` as the host mapped folders. In our [entry script](../share/container/entry_script_sh), installed to `/usr/local/bin/entry.sh` inside the container, switches to the `torrust` user created based upon the `USER_ID` environmental variable.

When running the container, you may use the `--env USER_ID="$(id -u)"` argument that gets the current user-id and passes to the container.

### Mapped Tree Structure

Using the standard mapping defined above produces this following mapped tree:

```s
storage/index/
├── lib
│   ├── database
│   │   └── sqlite3.db     => /var/lib/torrust/index/database/sqlite3.db [auto populated]
│   └── tls
│       ├── localhost.crt  => /var/lib/torrust/index/tls/localhost.crt [user supplied]
│       └── localhost.key  => /var/lib/torrust/index/tls/localhost.key [user supplied]
├── log                    => /var/log/torrust/index (future use)
└── etc
    ├── auth
    │   ├── private.pem    => /etc/torrust/index/auth/private.pem [auto generated on first boot]
    │   └── public.pem     => /etc/torrust/index/auth/public.pem  [auto generated on first boot]
    └── index.toml        => /etc/torrust/index/index.toml [auto populated]
```

> NOTE: you only need the `tls` directory and certificates in case you have enabled SSL.
>
> The `auth/` directory and RSA key pair are auto-generated on first boot by the
> container entry script. Sessions persist across restarts as long as the
> `/etc/torrust/index` volume is retained. To use your own keys, either
> pre-populate the volume before first boot or overwrite the generated files and
> restart.

## Building the Container

### Clone and Change into Repository

```sh
# Inside your dev folder
git clone https://github.com/torrust/torrust-index.git; cd torrust-index
```

### (Docker) Setup Context

Before starting, if you are using docker, it is helpful to reset the context to the default:

```sh
docker context use default
```

### (Docker) Build

```sh
# Release Mode
docker build --target release --tag torrust-index:release --file Containerfile .

# Debug Mode
docker build --target debug --tag torrust-index:debug --file Containerfile .
```

### (Podman) Build

```sh
# Release Mode
podman build --target release --tag torrust-index:release --file Containerfile .

# Debug Mode
podman build --target debug --tag torrust-index:debug --file Containerfile .
```

## Running the Container

### Basic Run

No arguments are needed for simply checking the container image works:

#### (Docker) Run Basic

```sh
# Release Mode
docker run -it torrust-index:release

# Debug Mode
docker run -it torrust-index:debug
```

#### (Podman) Run Basic

```sh
# Release Mode
podman run -it torrust-index:release

# Debug Mode
podman run -it torrust-index:debug
```

### Arguments

The arguments need to be placed before the image tag. i.e.

`run [arguments] torrust-index:release`

#### Environmental Variables:

Environmental variables are loaded through the `--env`, in the format `--env VAR="value"`.

The following environmental variables can be set:

- `TORRUST_INDEX_CONFIG_TOML_PATH` - The in-container path to the index configuration file, (default: `"/etc/torrust/index/index.toml"`).
- `TORRUST_INDEX_CONFIG_OVERRIDE_TRACKER__TOKEN` - Override of the admin token. If set, this value overrides any value set in the config.
- `TORRUST_INDEX_CONFIG_OVERRIDE_AUTH__PRIVATE_KEY_PATH` - Path to an RSA private key PEM file for JWT signing. Optional: without this, ephemeral auto-generated keys are used (sessions will not survive restarts).
- `TORRUST_INDEX_CONFIG_OVERRIDE_AUTH__PUBLIC_KEY_PATH` - Path to an RSA public key PEM file for JWT verification. Required when `PRIVATE_KEY_PATH` is set.
- `TORRUST_INDEX_CONFIG_OVERRIDE_AUTH__PRIVATE_KEY_PEM` - Inline RSA private key PEM string (alternative to file path). Optional: for persistent sessions.
- `TORRUST_INDEX_CONFIG_OVERRIDE_AUTH__PUBLIC_KEY_PEM` - Inline RSA public key PEM string (alternative to file path). Required when `PRIVATE_KEY_PEM` is set.
- `TORRUST_INDEX_DATABASE_DRIVER` - The database type used for the container, (options: `sqlite3`, `mysql`, default `sqlite3`). Please Note: This dose not override the database configuration within the `.toml` config file.
- `TORRUST_INDEX_CONFIG_TOML` - Load config from this environmental variable instead from a file, (i.e: `TORRUST_INDEX_CONFIG_TOML=$(cat index-index.toml)`).
- `USER_ID` - The user id for the runtime crated `torrust` user. Please Note: This user id should match the ownership of the host-mapped volumes, (default `1000`).
- `API_PORT` - The port for the index API. This should match the port used in the configuration, (default `3001`).
- `IMPORTER_API_PORT` - The port for the importer API. This should match the port used in the configuration, (default `3002`).

> NOTE: `API_PORT` and `IMPORTER_API_PORT` are runtime `ENV` values, not
> build-time `ARG`s. Overriding them at `docker run` / `podman run` time
> with `--env API_PORT=…` correctly reaches the application listener and
> the in-container `HEALTHCHECK`, but the `EXPOSE` directive in the
> `Containerfile` is evaluated at build time and bakes the *defaults*
> (`3001`, `3002`) into image metadata. Tools that read that metadata
> (`docker inspect`, `docker port`) will continue to report the defaults
> regardless of any `--env` override. Use `--publish host:container` to
> map whichever container port the application is actually listening on.

### Sockets

Socket ports used internally within the container can be mapped to with the `--publish` argument.

The format is: `--publish [optional_host_ip]:[host_port]:[container_port]/[optional_protocol]`, for example: `--publish 127.0.0.1:8080:80/tcp`.

The default ports can be mapped with the following:

```s
--publish 0.0.0.0:3001:3001/tcp
```

> NOTE: Inside the container it is necessary to expose a socket with the wildcard address `0.0.0.0` so that it may be accessible from the host. Verify that the configuration that the sockets are wildcard.

### Mapped Volumes

By default the container will install volumes for `/var/lib/torrust/index`, `/var/log/torrust/index`, and `/etc/torrust/index`, however for better administration it good to make these volumes host-mapped.

The argument to host-map volumes is `--volume`, with the format: `--volume=[host-src:]container-dest[:<options>]`.

The default mapping can be supplied with the following arguments:

```s
--volume ./storage/index/lib:/var/lib/torrust/index:Z \
--volume ./storage/index/log:/var/log/torrust/index:Z \
--volume ./storage/index/etc:/etc/torrust/index:Z \
```

Please not the `:Z` at the end of the podman `--volume` mapping arguments, this is to give read-write permission on SELinux enabled systemd, if this doesn't work on your system, you can use `:rw` instead.

## Complete Example

### With Docker

```sh
## Setup Docker Default Context
docker context use default

## Build Container Image
docker build --target release --tag torrust-index:release --file Containerfile .

## Setup Mapped Volumes
mkdir -p ./storage/index/lib/ ./storage/index/log/ ./storage/index/etc/

## Run Torrust Index Container Image
## Note: Without key path env vars, ephemeral auto-generated keys are used.
## For persistent sessions, supply your own RSA key pair:
##   --env TORRUST_INDEX_CONFIG_OVERRIDE_AUTH__PRIVATE_KEY_PATH="/var/lib/torrust/index/jwt/private.pem" \
##   --env TORRUST_INDEX_CONFIG_OVERRIDE_AUTH__PUBLIC_KEY_PATH="/var/lib/torrust/index/jwt/public.pem" \
docker run -it \
    --env TORRUST_INDEX_CONFIG_OVERRIDE_TRACKER__TOKEN="MySecretToken" \
    --env USER_ID="$(id -u)" \
    --publish 0.0.0.0:3001:3001/tcp \
    --volume ./storage/index/lib:/var/lib/torrust/index:Z \
    --volume ./storage/index/log:/var/log/torrust/index:Z \
    --volume ./storage/index/etc:/etc/torrust/index:Z \
    torrust-index:release
```

### With Podman

```sh
## Build Container Image
podman build --target release --tag torrust-index:release --file Containerfile .

## Setup Mapped Volumes
mkdir -p ./storage/index/lib/ ./storage/index/log/ ./storage/index/etc/

## Run Torrust Index Container Image
podman run -it \
    --env TORRUST_INDEX_CONFIG_OVERRIDE_TRACKER__TOKEN="MySecretToken" \
    --env USER_ID="$(id -u)" \
    --publish 0.0.0.0:3001:3001/tcp \
    --volume ./storage/index/lib:/var/lib/torrust/index:Z \
    --volume ./storage/index/log:/var/log/torrust/index:Z \
    --volume ./storage/index/etc:/etc/torrust/index:Z \
    torrust-index:release
```

## Runtime Image Notes

### Debug Image Healthcheck

The `debug` build target intentionally omits the `HEALTHCHECK` instruction
and does not include the `health_check` binary. This keeps the debug image
lightweight and avoids false-positive health signals during interactive
debugging sessions. If you need health-checking for a debug image, use an
external probe against the API port.

### Available Shell Commands (Busybox Subset)

The distroless runtime base ships only a minimal set of busybox applets
to reduce the attack surface:

- `sh`, `cat`, `ls`, `env`

Common utilities like `id`, `whoami`, `ps`, `grep`, and `wget` are **not**
available. If you need additional tools for operational debugging, use the
`debug` build target or exec into a sidecar container.

### Entry Script Debugging

The container entry script does not produce verbose output by default.
To enable shell tracing (`set -x`) for startup troubleshooting, set the
`DEBUG` environment variable:

```sh
--env DEBUG=1
```
