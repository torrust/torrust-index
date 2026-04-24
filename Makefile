# Top-level developer / operator entry points for the
# Compose split introduced by ADR-T-009 §8.3.
#
# Targets:
#
#   make up-dev   Plain `docker compose up`. Auto-loads
#                 `compose.override.yaml`, applying the dev
#                 sandbox extras (mailcatcher, permissive
#                 credential defaults, tty allocation). No
#                 validation — operators get whatever
#                 defaults the override provides.
#
#   make up-prod  Production-shaped invocation. Validates
#                 required credentials are set in the
#                 environment before any container starts,
#                 then runs `docker compose --file
#                 $(COMPOSE_FILE) up -d --wait` with the
#                 override excluded. Make propagates the
#                 recipe's exit code, so a failed
#                 healthcheck (`--wait`) or a missing
#                 credential (`:?required`) surfaces as a
#                 non-zero `make` exit.
#
# Validation logic uses POSIX `sh` with `set -u` and
# explicit `: "$${VAR:?message}"` per required variable so
# the file is dependency-free and shellcheck-clean.
#
# COMPOSE_FILE is overridable so acceptance tests (and
# operators with a non-default layout) can point at an
# alternate baseline without filesystem manipulation:
#
#   make up-prod COMPOSE_FILE=path/to/other.yaml
#
# Compose's own `COMPOSE_FILE` env var is ignored when
# `--file` is set on the command line, so the make-variable
# is the only reliable way to redirect the recipe.

.PHONY: up-dev up-prod _validate-prod-env

COMPOSE_FILE ?= compose.yaml

up-dev:
	docker compose up

# NOTE: The MySQL check is name-coupled to the compose
# service (`mysql:`). This is acceptable because it is
# defence-in-depth only — the config probe (exit 3 on
# empty connect_url) and the MySQL entrypoint (rejects
# empty root password) are the authoritative gates.
_validate-prod-env:
	@sh -uc '\
	  : "$${USER_ID:?required (numeric host UID owning ./storage)}" && \
	  : "$${TORRUST_INDEX_CONFIG_OVERRIDE_TRACKER__TOKEN:?required}" && \
	  : "$${TORRUST_INDEX_CONFIG_OVERRIDE_DATABASE__CONNECT_URL:?required}" && \
	  : "$${TORRUST_TRACKER_CONFIG_OVERRIDE_HTTP_API__ACCESS_TOKENS__ADMIN:?required}" && \
	  if grep -q "^[[:space:]]*mysql:" $(COMPOSE_FILE); then \
	    : "$${MYSQL_ROOT_PASSWORD:?required}"; \
	  fi'

up-prod: _validate-prod-env
	docker compose --file $(COMPOSE_FILE) up -d --wait
