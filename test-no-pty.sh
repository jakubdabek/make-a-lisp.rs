#!/usr/bin/env bash

make TEST_OPTS=--no-pty -C ../.. "test^rust2^${1}"
