#!/usr/bin/env bash

make TEST_OPTS=--no-pty --keep-going -C ../.. "${@/#/test^rust2^step}"
