#!/usr/bin/env bash
set -euo pipefail

tox -c deltachat-rpc-client -e py --devenv venv
venv/bin/pip install --upgrade pip
cargo install --locked --path deltachat-rpc-server/ --root "$PWD/venv" --debug
