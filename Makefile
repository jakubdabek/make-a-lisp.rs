DEBUG_STEPS := step_debug_eval step_debug_no_eval
STEPS := $(DEBUG_STEPS) step0_repl step1_read_print step2_eval step3_env step4_if_fn_do step5_tco step6_file

CARGO := $(or $(CARGO),cargo)

all: buildsteps $(STEPS)

buildsteps:
	$(CARGO) build --release --bins

step%: .FORCE
	$(CARGO) build --release --bin $@
	cp target/release/$@$(EXEC_EXT) ./$@

.PHONY: test buildsteps clean fmt .FORCE

test:
	$(CARGO) test --all-targets

clippy:
	$(CARGO) clippy --all-targets

clippy-fix:
	$(CARGO) clippy --all-targets --fix --allow-dirty

fmt:
	$(CARGO) +stable fmt

clean:
	$(CARGO) clean
	rm -f $(STEPS)
