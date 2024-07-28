#!/bin/bash

set -xe

SCRIPT=$(readlink -f $0)
SCRIPTPATH=`dirname $SCRIPT`

###########################################################
# FMT
###########################################################
cd ${SCRIPTPATH} && cargo fmt --check
#cd ${SCRIPTPATH}/examples/agc && cargo fmt --check

###########################################################
# CLIPPY
###########################################################
cd ${SCRIPTPATH} && cargo clippy --all-targets --all-features --workspace
# cd ${SCRIPTPATH}/examples/agc && cargo clippy --all-targets -- -D warnings

###########################################################
# Test
###########################################################
cd ${SCRIPTPATH} && cargo test --all-targets --all-features --workspace
# cd ${SCRIPTPATH}/examples/agc && cargo test --all-targets
