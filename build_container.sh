#!/bin/bash

set -e

# Choose tag
TAG=$1
if [ -z "$TAG" ]; then
    TAG="ollivarila/zeus-controller"
fi

# Build the container
echo "Building container with tag: $TAG"
docker build -t $1 .