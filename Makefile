STEPS = step0_repl step1_read_print

CARGO := $(or $(CARGO),cargo)

all: $(STEPS)

step%: .FORCE
	$(CARGO) build --release --bin $@
	cp target/release/$@$(EXEC_EXT) ./$@

.PHONY: test clean fmt .FORCE

test:
	$(CARGO) test --all-targets

fmt:
	$(CARGO) +stable fmt

clean:
	$(CARGO) clean
	rm -f $(STEPS)
