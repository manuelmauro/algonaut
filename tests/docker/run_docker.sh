#!/usr/bin/env bash
set -e

# reset test harness
rm -rf test-harness
rm -rf tests/features

git clone --single-branch --branch master https://github.com/algorand/algorand-sdk-testing.git test-harness

# copy feature files into project
mv test-harness/features tests/features

RUST_IMAGE=rust:1.58.1

echo "Building docker image from base \"$RUST_IMAGE\""

#build test environment
docker build -t rust-sdk-testing -f tests/docker/Dockerfile "$(pwd)"

# Start test harness environment
./test-harness/scripts/up.sh -p

docker run --network host rust-sdk-testing:latest
