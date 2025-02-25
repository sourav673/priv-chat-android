#!/bin/bash
#
# Run functional tests for Delta Chat core using the python bindings 
# and tox/pytest. 

set -e -x
shopt -s huponexit

# for core-building and python install step
export DCC_RS_TARGET=debug
export DCC_RS_DEV=`pwd`

cd python

cargo build -p deltachat_ffi --features jsonrpc

# remove and inhibit writing PYC files 
rm -rf tests/__pycache__
rm -rf src/deltachat/__pycache__
export PYTHONDONTWRITEBYTECODE=1

# run python tests (tox invokes pytest to run tests in python/tests)
#TOX_PARALLEL_NO_SPINNER=1 tox -e lint,doc
tox -e lint
tox -e doc
tox -e py -- "$@"
