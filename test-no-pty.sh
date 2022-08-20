#!/usr/bin/env bash

if [ $# -gt 0 ]
then
    targets=("${@/#/test^rust2^step}")
else
    targets=("test^rust2")
fi

make TEST_OPTS=--no-pty --keep-going -C ../.. "${targets[@]}"
