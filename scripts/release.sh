#!/usr/bin/env bash

#
# Copyright (c) 2023 Asim Ihsan.
#
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.
#
# SPDX-License-Identifier: MPL-2.0
#

# This script is used to release a new version of the project.
#
# Usage: ./scripts/release.sh <tag> [--force]
#
# The script will:
#   1. Accepts a tag input.
#   2. Checks if the tag exists.
#   3. Verifies that there is a passing GitHub workflow for the tag.
#   4. Uses cargo to cross-compile the project for Linux, Mac, Windows for x86 and arm64.
#   5. Creates a new release for the tag in draft mode and uploads the artifacts.
#   6. If the release already exists, provides a force flag to delete the draft release. If the release is published,
#      it should not be deleted.
#

set -euo pipefail

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"
source "${ROOT_DIR}/.env"

TAG=
FORCE=

while [[ $# -gt 0 ]]; do
  key="$1"
  case $key in
    --force)
      FORCE=true
      shift
      ;;
    *)
      TAG="$1"
      shift
      ;;
  esac
done

cross_targets=("aarch64-apple-darwin" "x86_64-apple-darwin")
for target in "${cross_targets[@]}"; do
  (cd "$ROOT_DIR" && cargo build --all --release --target "$target")
done

mkdir -p "${ROOT_DIR}/target/universal/release"
lipo -create \
  -output "${ROOT_DIR}/target/universal/release/code-digest" \
  "${ROOT_DIR}/target/aarch64-apple-darwin/release/code-digest" \
  "${ROOT_DIR}/target/x86_64-apple-darwin/release/code-digest"

"${SCRIPT_DIR}"/notarize.sh \
  --binary-path "${ROOT_DIR}/target/universal/release/code-digest" \
  --output-zip-path "${ROOT_DIR}target/universal/release/code-digest-macos.zip" \
  --developer-id "$DEVELOPER_ID" \
  --apple-id "$APPLE_ID" \
  --app-specific-password "$APP_SPECIFIC_PASSWORD"

if [[ -z "$TAG" ]]; then
  echo "Error: No tag specified."
  exit 1
fi

# Check if the tag exists
if ! git rev-parse "$TAG" >/dev/null 2>&1; then
  echo "Error: Tag $TAG does not exist."
  exit 1
fi

# If the current Git clone is dirty, abort
if ! git diff-index --quiet HEAD --; then
  echo "Error: Git clone is dirty. Commit all changes before releasing."
  exit 1
fi

# If the current Git clone is not at the tag, abort
if ! git describe --exact-match --tags HEAD >/dev/null 2>&1; then
  echo "Error: Git clone is not at the tag $TAG. Checkout the tag before releasing."
  exit 1
fi

# Check if there is a passing GitHub workflow for the tag
if ! gh run list --limit 1 --workflow ".github/workflows/release.yml" --branch "$TAG" --json conclusion -q '.[0].conclusion' | grep -q "success"; then
  echo "Error: No passing GitHub workflow found for tag $TAG."
  exit 1
fi

# Check if the release already exists
release_id=$(gh release view "$TAG" --json id -q '.id' 2>/dev/null || echo "")

if [[ -n "$release_id" ]]; then
  if [[ "$FORCE" == "true" ]]; then
    # Delete the draft release
    gh release delete "$TAG" --yes
  else
    echo "Error: Release for tag $TAG already exists. Use --force to delete the draft release."
    exit 1
  fi
fi

# Create a new release in draft mode
gh release create "$TAG" --title "Release $TAG" --notes "Release notes for $TAG" --draft

# Upload the artifacts
artifact="${ROOT_DIR}target/universal/release/code-digest-macos.zip"
gh release upload "$TAG" "$artifact" --clobber

echo "Release $TAG created successfully."
