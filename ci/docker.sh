#!/bin/sh
. ./ci/preamble.sh
image=ghcr.io/igankevich/human-units-ci:latest
docker build --tag $image - <ci/Dockerfile
docker push $image
