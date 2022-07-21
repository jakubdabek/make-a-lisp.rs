STEPS = step0_repl

CARGO := $(or $(CARGO),cargo)

all: $(STEPS)

step%: .FORCE
	$(CARGO) build --release --bin $@
	cp target/release/$@$(EXEC_EXT) ./$@

.PHONY: clean fmt .FORCE

fmt:
	$(CARGO) fmt

clean:
	$(CARGO) clean
	rm -f $(STEPS)
