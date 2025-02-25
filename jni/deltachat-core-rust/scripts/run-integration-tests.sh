#!/bin/bash

# Small helper to easily run integration tests locally for development
# purposes.  Any arguments are passed straight to tox.  E.g. to run
# only one environment run with:
#
#   scripts/run-integration-tests.sh -e py35
#
# To also run with `pytest -x` use:
#
#   scripts/run-integration-tests.sh -e py35 -- -x

export DCC_RS_DEV=$(pwd)
export DCC_RS_TARGET=${DCC_RS_TARGET:-release}

if [ $DCC_RS_TARGET = 'release' ]; then
    cargo build -p deltachat_ffi --release
else
    cargo build -p deltachat_ffi
fi
if [ $? != 0 ]; then
    exit $?
fi

pushd python
tox "$@"
ret=$?
popd
exit $ret
