#!/usr/bin/env bash

# Generate bindings
openapi-generator-cli generate \
    -g rust \
    -o . \
    -i https://api.jellyfin.org/openapi/jellyfin-openapi-stable.json \
    --library reqwest \
    --package-name jellyfin

# Patch generation errors
patch -p1 < fix.patch