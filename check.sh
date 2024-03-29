#!/bin/bash

set -xe

SCRIPT=$(readlink -f $0)
SCRIPTPATH=`dirname $SCRIPT`

###########################################################
# FMT
###########################################################
cd ${SCRIPTPATH} && cargo fmt --check

###########################################################
# CLIPPY
###########################################################
cd ${SCRIPTPATH} && cargo clippy --all-targets --all-features --workspace 

###########################################################
# Test
###########################################################
cd ${SCRIPTPATH} && cargo test --all-targets --all-features --workspace 
