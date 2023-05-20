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

set -euo pipefail

DEVELOPER_ID=
APPLE_ID=
APP_SPECIFIC_PASSWORD=
BINARY_PATH=
OUTPUT_ZIP_PATH=

while [[ $# -gt 0 ]]; do
  key="$1"
  case $key in
    --binary-path)
      BINARY_PATH="$2"
      shift
      shift
      ;;
    --output-zip-path)
      OUTPUT_ZIP_PATH="$2"
      shift
      shift
      ;;
    --developer-id)
      DEVELOPER_ID="$2"
      shift
      shift
      ;;
    --apple-id)
      APPLE_ID="$2"
      shift
      shift
      ;;
    --app-specific-password)
      APP_SPECIFIC_PASSWORD="$2"
      shift
      shift
      ;;
    *)
      echo "Unknown option $key"
      exit 1
      ;;
  esac
done

if [[ -z "$BINARY_PATH" ]]; then
  echo "Error: --binary-path not specified"
  exit 1
fi

if [[ ! -f "$BINARY_PATH" ]]; then
  echo "Error: $BINARY_PATH does not exist"
  exit 1
fi

if [[ -z "$OUTPUT_ZIP_PATH" ]]; then
  echo "Error: --output-zip-path not specified"
  exit 1
fi

if [[ -z "$DEVELOPER_ID" ]]; then
  echo "Error: --developer-id not specified"
  exit 1
fi

if [[ -z "$APPLE_ID" ]]; then
  echo "Error: --apple-id not specified"
  exit 1
fi

if [[ -z "$APP_SPECIFIC_PASSWORD" ]]; then
  echo "Error: --app-specific-password not specified"
  exit 1
fi

BINARY_NAME="$(basename "$BINARY_PATH")"
BINARY_DIR="$(dirname "$BINARY_PATH")"

pushd "$BINARY_DIR" > /dev/null || exit 1
trap "popd > /dev/null" EXIT

# Sign the binary
echo "Signing $BINARY_NAME"
codesign --deep --force --verify --verbose --timestamp --options runtime --sign "$DEVELOPER_ID" "$BINARY_NAME"

# Compress the binary to OUTPUT_ZIP_PATH
echo "Compressing $BINARY_NAME to $OUTPUT_ZIP_PATH"

# Use --keepParents for macOS apps, but for just a binary we don't need it.
ditto -c -k --sequesterRsrc "$BINARY_NAME" "$OUTPUT_ZIP_PATH"

# Get your team ID from WWDRTeam
echo "Getting team ID"
TEAM_ID="$(xcrun altool --list-providers -u "$APPLE_ID" -p "${APP_SPECIFIC_PASSWORD}" | tail -2 | awk '{print $NF}')"

# Start notarization, this will return a request UUID
echo "Starting notarization"
xcrun notarytool submit \
  --apple-id "$APPLE_ID" \
  --password "${APP_SPECIFIC_PASSWORD}" \
  --team-id "$TEAM_ID" \
  --wait \
  "$OUTPUT_ZIP_PATH"
