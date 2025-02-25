#!/bin/bash 

BUILD_ID=${1:?specify build ID}

SSHTARGET=${SSHTARGET-ci@b1.delta.chat}
BUILDDIR=ci_builds/$BUILD_ID

set -e

echo "--- Copying files to $SSHTARGET:$BUILDDIR"

ssh -oBatchMode=yes -oStrictHostKeyChecking=no $SSHTARGET mkdir -p "$BUILDDIR"
git ls-files >.rsynclist
rsync --delete --files-from=.rsynclist -az ./ "$SSHTARGET:$BUILDDIR"

echo "--- Running Rust tests remotely"

ssh $SSHTARGET <<_HERE
    set +x -e
    # make sure all processes exit when ssh dies
    shopt -s huponexit
    export RUSTC_WRAPPER=\`which sccache\`
    cd $BUILDDIR
    export TARGET=x86_64-unknown-linux-gnu
    export RUSTC_WRAPPER=sccache

    bash scripts/run-rust-test.sh
_HERE

