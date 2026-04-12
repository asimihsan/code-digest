#!/usr/bin/env bash

set -euo pipefail

if [[ "${PROJECT_ENV_DEBUG:-0}" == "1" ]]; then
  echo "PROJECT_ROOT=${MISE_PROJECT_ROOT:-$(pwd)}" >&2
fi

export RUST_BACKTRACE="${RUST_BACKTRACE:-1}"
