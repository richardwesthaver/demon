#!/bin/bash
# handy script for updating demon crate and inter-dependency versions
# this is probably safer outside of a Makefile.toml..
# see https://github.com/amethyst/amethyst/blob/master/script/set-version
VERSION="$1"

if [[ ${VERSION} == "" ]] ; then 
  echo -e "Usage: set-version X.Y.Z\n\nUpdates all Cargo.toml files with a new version."
  exit 3
fi

function die() {
    echo "$1"
    exit 2
}

which rg > /dev/null || die "Please 'cargo install ripgrep'"

# caret means only at start of line, so avoids upstream dependencies. This does mean we can't use the [dependencies.$name] syntax in Cargo.toml though.
CARGO_TOML_FILES=$(rg '^version =' -t toml -l)

for file in $CARGO_TOML_FILES ; do
    sed -E \
    -e "s/^version = \".*\"\$/version = \"${VERSION}\"/" \
    -e "s/(demon_.*\", )version = \"[0-9]+.[0-9]+.[0-9]+\"/\\1version = \"${VERSION}\"/g" \
    -e "s/^(demon = \\{ path = \"\.\.\", )version = \"[0-9]+.[0-9]+.[0-9]+\"/\\1version = \"${VERSION}\"/g" \
    -i "${file}"
done
