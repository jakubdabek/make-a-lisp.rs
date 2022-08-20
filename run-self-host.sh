#!/usr/bin/env bash

make MAL_IMPL=rust2 TEST_OPTS=--no-pty --keep-going -C ../.. build^rust2^stepA "${1/#/repl^mal^step}"
