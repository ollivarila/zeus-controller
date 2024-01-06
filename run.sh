#!/bin/bash

set -e

export RUST_LOG=info
export TEMPLATE_PATH=templates

systemfd --no-pid -s http::3001 -- cargo watch -x run