# syntax=docker/dockerfile:latest

# Torrust Index

## Builder Image
FROM rust:trixie AS chef
WORKDIR /tmp
RUN curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/v1.18.1/install-from-binstall-release.sh | bash
RUN cargo binstall --no-confirm --locked cargo-chef cargo-nextest

## Tester Image
FROM rust:slim-trixie AS tester
WORKDIR /tmp

RUN apt-get update; apt-get install -y curl sqlite3; apt-get autoclean
RUN curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/v1.18.1/install-from-binstall-release.sh | bash
RUN cargo binstall --no-confirm --locked cargo-nextest imdl

COPY ./share/ /app/share/torrust
RUN mkdir -p /app/share/torrust/default/database/; \
    sqlite3 /app/share/torrust/default/database/index.sqlite3.db  "VACUUM;"

## jq donor (pristine base, no user code)
FROM rust:slim-trixie AS jq_donor
RUN apt-get update && \
    apt-get install -y --no-install-recommends jq && \
    rm -rf /var/lib/apt/lists/*

## Su Exe Compile
FROM docker.io/library/gcc:trixie AS gcc
COPY ./contrib/dev-tools/su-exec/ /usr/local/src/su-exec/
RUN cc -Wall -Werror -g /usr/local/src/su-exec/su-exec.c -o /usr/local/bin/su-exec; chmod +x /usr/local/bin/su-exec


## Chef Prepare (look at project and see what we need)
FROM chef AS recipe
WORKDIR /build/src
COPY . /build/src
RUN cargo chef prepare --recipe-path /build/recipe.json


## Cook (debug)
FROM chef AS dependencies_debug
WORKDIR /build/src
COPY --from=recipe /build/recipe.json /build/recipe.json
RUN cargo chef cook --workspace --all-targets --all-features --recipe-path /build/recipe.json
RUN cargo nextest archive --workspace --all-targets --all-features --archive-file /build/temp.tar.zst ; rm -f /build/temp.tar.zst

## Cook (release)
FROM chef AS dependencies
WORKDIR /build/src
COPY --from=recipe /build/recipe.json /build/recipe.json
RUN cargo chef cook --workspace --all-targets --all-features --recipe-path /build/recipe.json --release
RUN cargo nextest archive --workspace --all-targets --all-features --archive-file /build/temp.tar.zst --release ; rm -f /build/temp.tar.zst


## Build Archive (debug)
FROM dependencies_debug AS build_debug
WORKDIR /build/src
COPY . /build/src
RUN cargo nextest archive --workspace --all-targets --all-features --archive-file /build/torrust-index-debug.tar.zst

## Build Archive (release)
FROM dependencies AS build
WORKDIR /build/src
COPY . /build/src
RUN cargo nextest archive --workspace --all-targets --all-features --archive-file /build/torrust-index.tar.zst --release


# Extract and Test (debug)
FROM tester AS test_debug
WORKDIR /test
COPY . /test/src/
COPY --from=build_debug \
  /build/torrust-index-debug.tar.zst \
  /test/torrust-index-debug.tar.zst
RUN cargo nextest run --workspace-remap /test/src/ --extract-to /test/src/ --no-run --archive-file /test/torrust-index-debug.tar.zst
RUN cargo nextest run --workspace-remap /test/src/ --target-dir-remap /test/src/target/ --cargo-metadata /test/src/target/nextest/cargo-metadata.json --binaries-metadata /test/src/target/nextest/binaries-metadata.json

RUN mkdir -p /app/bin/; \
  cp -l /test/src/target/debug/torrust-index /app/bin/torrust-index; \
  cp -l /test/src/target/debug/torrust-index-health-check /app/bin/torrust-index-health-check; \
  cp -l /test/src/target/debug/torrust-index-auth-keypair /app/bin/torrust-index-auth-keypair
# Phase 4: per-binary modes. Application binary stays
# world-executable; root-phase-only helpers (health-check,
# auth-keypair, config-probe — added in Phase 6) tighten to
# root-only (0500 root:root). Same posture as busybox,
# su-exec, jq. The healthcheck binary is invoked from
# HEALTHCHECK, which runs as root (no --user in the
# directive), so 0500 is sufficient.
RUN chown -R root:root /app; chmod -R u=rw,go=r,a+X /app; \
    chmod 0755 /app/bin/torrust-index; \
    chown 0:0  /app/bin/torrust-index-health-check \
               /app/bin/torrust-index-auth-keypair; \
    chmod 0500 /app/bin/torrust-index-health-check \
               /app/bin/torrust-index-auth-keypair

# Extract and Test (release)
FROM tester AS test
WORKDIR /test
COPY . /test/src
COPY --from=build \
  /build/torrust-index.tar.zst \
  /test/torrust-index.tar.zst
RUN cargo nextest run --workspace-remap /test/src/ --extract-to /test/src/ --no-run --archive-file /test/torrust-index.tar.zst
RUN cargo nextest run --workspace-remap /test/src/ --target-dir-remap /test/src/target/ --cargo-metadata /test/src/target/nextest/cargo-metadata.json --binaries-metadata /test/src/target/nextest/binaries-metadata.json

RUN mkdir -p /app/bin/; \
  cp -l /test/src/target/release/torrust-index /app/bin/torrust-index; \
  cp -l /test/src/target/release/torrust-index-health-check /app/bin/torrust-index-health-check; \
  cp -l /test/src/target/release/torrust-index-auth-keypair /app/bin/torrust-index-auth-keypair
# Phase 4: per-binary modes (see test_debug above for rationale).
# The healthcheck binary is invoked from HEALTHCHECK, which
# runs as root (no --user in the directive), so 0500 is
# sufficient.
RUN chown -R root:root /app; chmod -R u=rw,go=r,a+X /app; \
    chmod 0755 /app/bin/torrust-index; \
    chown 0:0  /app/bin/torrust-index-health-check \
               /app/bin/torrust-index-auth-keypair; \
    chmod 0500 /app/bin/torrust-index-health-check \
               /app/bin/torrust-index-auth-keypair


## ── Runtime asset bundle (base-agnostic) ─────────────────────
# The lean release base does not ship busybox at all; the
# :debug variant does. Use the :debug image as a "donor" we
# extract a single root-only busybox binary from for the
# release base.
FROM gcr.io/distroless/cc-debian13:debug AS busybox_donor

# Preflight: assert the donor still ships a real busybox ELF
# at the path we copy from, and that its `install` applet
# supports `-D` (the entry script's `inst()` helper depends on
# it). Cheap insurance against a future base reshuffle.
FROM busybox_donor AS busybox_preflight
RUN ["/busybox/sh", "-c", \
     "test -f /busybox/busybox \
      && /busybox/busybox --help >/dev/null \
      && /busybox/busybox install --help 2>&1 | grep -q -- '-D'"]

# Minimal /etc account files generated locally so adduser
# works regardless of base. The base image's own
# /etc/nsswitch.conf (which includes `hosts: files dns`) is
# deliberately left alone — the application makes outbound
# connections where host-name resolution must work.
FROM busybox_preflight AS etc_seed
# /etc/profile is seeded as an empty file so the debug image's
# `ENV ENV=/etc/profile` points at a real path. Operators who
# bind-mount a richer profile over it get the expected
# behaviour; absent that, busybox `sh` sources an empty file
# and continues silently rather than warning on every
# invocation.
RUN ["/busybox/sh", "-c", \
     "mkdir -p /seed/etc && \
      printf 'root:x:0:0:root:/:/bin/sh\\n'    > /seed/etc/passwd && \
      printf 'root:x:0:\\n'                    > /seed/etc/group  && \
      : > /seed/etc/profile"]

# Preflight: assert `adduser -D` works without /etc/shadow.
# The entry script runs `adduser -D -s /bin/sh -u $USER_ID
# torrust` at first boot against the etc_seed layout (passwd
# + group, no shadow). Busybox adduser behaviour when shadow
# is absent varies by version; this stage catches regressions
# at build time rather than first boot.
FROM busybox_donor AS adduser_preflight
COPY --from=etc_seed /seed/etc/passwd /etc/passwd
COPY --from=etc_seed /seed/etc/group  /etc/group
# Busybox `adduser` rejects UIDs outside 0..60000 (well below
# the conventional `nobody` value of 65534). Use a value in
# range that is still well above `USER_ID=1000` so the
# preflight cannot collide with a realistic runtime user.
RUN ["/busybox/sh", "-c", \
     "/busybox/adduser -D -s /bin/sh -u 59999 testuser \
      && /busybox/grep -q '^testuser:' /etc/passwd \
      && /busybox/test -d /home/testuser"]

FROM scratch AS runtime_assets
COPY --from=etc_seed --chmod=0644 --chown=0:0 /seed/etc/ /etc/
# Single busybox binary, root-only. Copy the file directly
# (not via the /busybox/sh symlink) to avoid depending on the
# donor's symlink layout.
COPY --from=busybox_preflight --chmod=0700 --chown=0:0 \
    /busybox/busybox /bin/busybox
COPY --from=gcc --chmod=0700 --chown=0:0 \
    /usr/local/bin/su-exec  /bin/su-exec
COPY --chmod=0555 --chown=0:0 \
    ./share/container/entry_script_sh  /usr/local/bin/entry.sh

## ── Preflight gate (aggregates all donor-validation stages) ──
# Both runtime bases COPY from this stage, creating an
# explicit BuildKit dependency edge that prevents any
# preflight from being pruned regardless of which image
# variant is built.
FROM scratch AS preflight_gate
COPY --from=busybox_preflight /etc/passwd /tmp/.busybox-ok
COPY --from=adduser_preflight /etc/passwd /tmp/.adduser-ok

## ── Runtime base: release (root-only curated subset) ─────────
FROM gcr.io/distroless/cc-debian13 AS runtime_release
# Note: distroless cc-debian13 is usrmerged — `/bin` is a
# symlink to `/usr/bin`, so a recursive `COPY / /` from a
# stage that ships `/bin/` as a real directory fails with
# "cannot copy to non-directory". Copy each curated path
# individually instead so BuildKit resolves the symlink.
COPY --from=runtime_assets /etc/passwd  /etc/passwd
COPY --from=runtime_assets /etc/group   /etc/group
COPY --from=runtime_assets /etc/profile /etc/profile
COPY --from=runtime_assets /bin/busybox            /usr/bin/busybox
COPY --from=runtime_assets /bin/su-exec            /usr/bin/su-exec
COPY --from=runtime_assets /usr/local/bin/entry.sh /usr/local/bin/entry.sh
COPY --from=preflight_gate /tmp/.adduser-ok /tmp/.preflight-sentinel
# Pin PATH so a future base-image change cannot silently break
# the entry script's bare-name lookups.
ENV PATH=/usr/local/bin:/bin:/usr/bin:/sbin
# Materialise the curated applet set as symlinks to the
# single root-only busybox binary. Symlinks are created in
# `/usr/bin/` (the canonical usrmerged location); `/bin/<a>`
# resolves to the same path via the base's `/bin → /usr/bin`
# symlink. The applet list must match the curated applet
# reference in ADR-T-009 §D4 — update both in the same change.
RUN ["/usr/bin/busybox", "sh", "-c", \
     "for a in sh adduser addgroup install mkdir dirname chown chmod tr mktemp cat printf rm echo grep; do \
        /usr/bin/busybox ln -s busybox /usr/bin/$a; \
      done && rm -f /tmp/.preflight-sentinel"]

## ── Runtime base: debug (full busybox on PATH) ───────────────
FROM gcr.io/distroless/cc-debian13:debug AS runtime_debug
COPY --from=etc_seed --chmod=0644 --chown=0:0 /seed/etc/ /etc/
COPY --from=preflight_gate /tmp/.adduser-ok /tmp/.preflight-sentinel
# Pull su-exec from runtime_assets (which already copies it
# from gcc with the correct mode/ownership) so there is a
# single source for the compiled binary regardless of base.
COPY --from=runtime_assets /bin/su-exec /bin/su-exec
COPY --chmod=0555 --chown=0:0 \
    ./share/container/entry_script_sh  /usr/local/bin/entry.sh
# Materialise /bin/sh → /busybox/sh so root's recorded login
# shell in /etc/passwd resolves correctly. The release base
# creates the same symlink as part of its curated-applet
# loop; the debug base needs an explicit one because
# /bin/busybox doesn't exist here.
RUN ["/busybox/sh", "-c", \
     "/busybox/ln -s /busybox/sh /bin/sh && rm -f /tmp/.preflight-sentinel"]
ENV PATH=/usr/local/bin:/busybox:/bin:/usr/bin:/sbin

## Torrust-Index (debug)
FROM runtime_debug AS debug
ENV TORRUST_INDEX_CONFIG_TOML_PATH=/etc/torrust/index/index.toml \
    TORRUST_INDEX_DATABASE_DRIVER=sqlite3 \
    USER_ID=1000 \
    API_PORT=3001 \
    IMPORTER_API_PORT=3002 \
    TZ=Etc/UTC \
    ENV=/etc/profile \
    RUNTIME=debug
EXPOSE 3001/tcp 3002/tcp
VOLUME ["/var/lib/torrust/index","/var/log/torrust/index","/etc/torrust/index"]
COPY --from=test_debug /app/ /usr/
# jq binary for entry-script JSON consumption (§2.2 step 4).
# Root-only (0500) — same posture as busybox and su-exec.
COPY --from=jq_donor --chmod=0500 --chown=0:0 /usr/bin/jq /usr/bin/jq
ENTRYPOINT ["/usr/local/bin/entry.sh"]
HEALTHCHECK --interval=5s --timeout=5s --start-period=3s --retries=3 \
  CMD /usr/bin/torrust-index-health-check "http://localhost:${API_PORT}/health_check" \
   && /usr/bin/torrust-index-health-check "http://localhost:${IMPORTER_API_PORT}/health_check"
# Default CMD matches release so the debug image is a drop-in
# replacement; operators can override with `sh` (or any other
# applet on PATH) at `docker run` / compose time.
CMD ["/usr/bin/torrust-index"]

## Torrust-Index (release) (default)
FROM runtime_release AS release
ENV TORRUST_INDEX_CONFIG_TOML_PATH=/etc/torrust/index/index.toml \
    TORRUST_INDEX_DATABASE_DRIVER=sqlite3 \
    USER_ID=1000 \
    API_PORT=3001 \
    IMPORTER_API_PORT=3002 \
    TZ=Etc/UTC \
    RUNTIME=release
EXPOSE 3001/tcp 3002/tcp
VOLUME ["/var/lib/torrust/index","/var/log/torrust/index","/etc/torrust/index"]
COPY --from=test /app/ /usr/
# jq binary for entry-script JSON consumption (§2.2 step 4).
# Root-only (0500) — same posture as busybox and su-exec.
COPY --from=jq_donor --chmod=0500 --chown=0:0 /usr/bin/jq /usr/bin/jq
ENTRYPOINT ["/usr/local/bin/entry.sh"]
HEALTHCHECK --interval=5s --timeout=5s --start-period=3s --retries=3 \
  CMD /usr/bin/torrust-index-health-check "http://localhost:${API_PORT}/health_check" \
   && /usr/bin/torrust-index-health-check "http://localhost:${IMPORTER_API_PORT}/health_check"
CMD ["/usr/bin/torrust-index"]
