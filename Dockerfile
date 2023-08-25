# syntax=docker/dockerfile:latest

# Torrust Index Backend

## Builder Image
FROM rust:latest as chef
WORKDIR /tmp
RUN curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
RUN cargo binstall --no-confirm cargo-chef cargo-nextest


## Tester Image
FROM rust:slim as tester
WORKDIR /tmp
### (fixme) https://github.com/cargo-bins/cargo-binstall/issues/1252
RUN apt-get update; apt-get install -y curl; apt-get autoclean
RUN curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
RUN cargo binstall --no-confirm cargo-nextest imdl


## Chef Prepare (look at project and see wat we need)
FROM chef AS recipe
WORKDIR /build/src
COPY . /build/src
RUN cargo chef prepare --recipe-path /build/recipe.json


## Cook (debug)
FROM chef AS dependencies_debug
WORKDIR /build/src
COPY --from=recipe /build/recipe.json /build/recipe.json
RUN cargo chef cook --tests --benches --examples --workspace --all-targets --all-features --recipe-path /build/recipe.json
RUN cargo nextest archive --tests --benches --examples --workspace --all-targets --all-features --archive-file /build/temp.tar.zst ; rm /build/temp.tar.zst

## Cook (release)
FROM chef AS dependencies
WORKDIR /build/src
COPY --from=recipe /build/recipe.json /build/recipe.json
RUN cargo chef cook --tests --benches --examples --workspace --all-targets --all-features --recipe-path /build/recipe.json --release
RUN cargo nextest archive --tests --benches --examples --workspace --all-targets --all-features --archive-file /build/temp.tar.zst --release  ; rm /build/temp.tar.zst


## Build Archive (debug)
FROM dependencies_debug AS build_debug
WORKDIR /build/src
COPY . /build/src
RUN cargo nextest archive --tests --benches --examples --workspace --all-targets --all-features --archive-file /build/torrust-index-backend-debug.tar.zst

## Build Archive (release)
FROM dependencies AS build
WORKDIR /build/src
COPY . /build/src
RUN cargo nextest archive --tests --benches --examples --workspace --all-targets --all-features --archive-file /build/torrust-index-backend.tar.zst --release


# Extract and Test (debug)
FROM tester as test_debug
WORKDIR /test
COPY . /test/src
COPY --from=build_debug \
  /build/torrust-index-backend-debug.tar.zst \
  /test/torrust-index-backend-debug.tar.zst
RUN mkdir -p /test/test
RUN cargo nextest run --workspace-remap /test/src/ --extract-to /test/src/ --no-run --archive-file /test/torrust-index-backend-debug.tar.zst
RUN cargo nextest run --workspace-remap /test/src/ --target-dir-remap /test/src/target/ --cargo-metadata /test/src/target/nextest/cargo-metadata.json --binaries-metadata /test/src/target/nextest/binaries-metadata.json

RUN mkdir -p /app/bin/; cp -l /test/src/target/debug/main     /app/bin/torrust-index-backend
RUN mkdir -p /app/bin/; cp -l /test/src/target/debug/upgrader /app/bin/torrust-index-backend-upgrader
RUN mkdir -p /app/bin/; cp -l /test/src/target/debug/importer /app/bin/torrust-index-backend-importer

# RUN mkdir /app/lib/; cp -l $(realpath $(ldd /app/bin/torrust-index-backend | grep "libz\.so\.1" | awk '{print $3}')) /app/lib/libz.so.1

RUN chown -R root:root /app
RUN chmod -R u=rw,go=r,a+X /app
RUN chmod -R a+x /app/bin

# Extract and Test (release)
FROM tester as test
WORKDIR /test
COPY . /test/src
COPY --from=build \
  /build/torrust-index-backend.tar.zst \
  /test/torrust-index-backend.tar.zst
RUN cargo nextest run --workspace-remap /test/src/ --extract-to /test/src/ --no-run --archive-file /test/torrust-index-backend.tar.zst
RUN cargo nextest run --workspace-remap /test/src/ --target-dir-remap /test/src/target/ --cargo-metadata /test/src/target/nextest/cargo-metadata.json --binaries-metadata /test/src/target/nextest/binaries-metadata.json

RUN mkdir -p /app/bin/; cp -l /test/src/target/release/main     /app/bin/torrust-index-backend
RUN mkdir -p /app/bin/; cp -l /test/src/target/release/upgrader /app/bin/torrust-index-backend-upgrader
RUN mkdir -p /app/bin/; cp -l /test/src/target/release/importer /app/bin/torrust-index-backend-importer

# RUN mkdir -p /app/lib/; cp -l $(realpath $(ldd /app/bin/torrust-index-backend | grep "libz\.so\.1" | awk '{print $3}')) /app/lib/libz.so.1

RUN chown -R root:root /app
RUN chmod -R u=rw,go=r,a+X /app
RUN chmod -R a+x /app/bin


## Torrust-Index-Backend (debug)
FROM gcr.io/distroless/cc:debug as index_backend_debug

RUN ["/busybox/cp", "-sp", "/busybox/sh", "/bin/sh"]
ENV ENV=/etc/profile

ARG USER_ID=1000
ARG USER_NAME=appuser
ARG API_PORT=3001

ENV USER_ID=${USER_ID}
ENV USER_NAME=${USER_NAME}
ENV API_PORT=${API_PORT}
ENV TZ=Etc/UTC

EXPOSE ${API_PORT}/tcp

COPY --from=test_debug /app/ /usr/

COPY ./docker/motd.debug /etc/motd

RUN echo '[ ! -z "$TERM" -a -r /etc/motd ] && cat /etc/motd' >> /etc/profile

WORKDIR /home/${USER_NAME}
RUN adduser --disabled-password --uid "${USER_ID}" "${USER_NAME}"
USER "${USER_NAME}":"${USER_NAME}"

RUN env

## Torrust-Index-Backend (release) (default)
FROM gcr.io/distroless/cc:nonroot as index_backend
COPY --from=gcr.io/distroless/cc:debug /busybox/wget /usr/bin/wget
COPY --from=test /app/ /usr/

ARG API_PORT=3001

ENV API_PORT=${API_PORT}
ENV TZ=Etc/UTC

EXPOSE ${API_PORT}/tcp

HEALTHCHECK CMD ["/usr/bin/wget", "--no-verbose", "--tries=1", "--spider", "localhost:${API_PORT}"]

CMD ["/usr/bin/torrust-index-backend"]
