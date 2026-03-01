#!/usr/bin/env bash
tmpdir=$(mktemp -d)
# Mac is stupid so we can't use transform in tar nor -executable in find
find ./ -type f -perm -111 -iname "grustonnet-*" -exec cp {} "$tmpdir/" \;
# Stupid but makes sure it works everywhere
pushd "$tmpdir" || exit 1
tar -czf "$RELEASE_TAR" ./*
popd || exit 1
mv "$tmpdir"/"$RELEASE_TAR" ./
rm -rf "$tmpdir"
