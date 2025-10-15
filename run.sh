#!/bin/bash -x

source ./env

certs/gen-certs.sh

target/release/photo-mcp-server
