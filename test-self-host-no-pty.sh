#!/usr/bin/env bash

if [ $# -gt 0 ]
then
    targets=("${@/#/test^mal^step}")
else
    targets=("test^mal")
fi

make MAL_IMPL=rust2 TEST_OPTS=--no-pty --keep-going -C ../.. build^rust2^stepA "${targets[@]}"
