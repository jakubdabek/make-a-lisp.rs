# mal.rs

An interpreter for a Lisp dialect based on the [mal - Make a Lisp](https://github.com/kanaka/mal) project.

Everything has been reimplemented in Rust from scratch,
loosely based on the [steps outlined in the original repository](https://github.com/kanaka/mal/blob/master/process/guide.md).
Module organization doesn't follow the structure of the original,
so it's not been submitted to the main repository.

## Notable features

- Variables
- Basic structures - lists, vectors, hash maps
- Function objects, closures
- Variadic function arguments
- Quoting (`'(1 2 3)`)
- Macros
- Guaranteed Tail-Call Optimization (TCO)
- Exceptions
- Capable of self-hosting (running an interpreter written in the `mal` language itself)
- `stdin` and `stdout`
- String manipulation
- File reading

## How to run

```sh
## clone the original mal repository
$ git clone https://github.com/kanaka/mal
## clone this repository into the `impls` folder as `rust2`
$ cd mal/impls
$ git clone https://github.com/jakubdabek/make-a-lisp.rs rust2

## run tests for each step with the scripts included
$ cd rust2

## if using WSL and rust is only installed on Windows:
# $ source ./wsl.env.sh

$ ./test-no-pty.sh          # run all tests
$ ./test-no-pty.sh 3 5 7    # run tests for steps 3, 5 and 7

## testing self-hosting capabilities
$ ./test-self-host-no-pty.sh        # run all tests
$ ./test-self-host-no-pty.sh 3 5 7  # run tests for steps 3, 5 and 7

## running self-host REPL
$ ./run-self-host.sh 5  # run REPL from step5
```

