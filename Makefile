DEBUG_STEPS := step_debug_eval step_debug_no_eval
STEPS := \
	step0_repl \
	step1_read_print \
	step2_eval \
	# end of STEPS

REPEATING_STEPS := \
	step3_env \
	step4_if_fn_do \
	step5_tco \
	step6_file \
	step7_quote \
	step8_macros \
	step9_try \
	stepA_mal \
	# end of REPEATING_STEPS

ALL_STEPS := $(DEBUG_STEPS) $(STEPS) $(REPEATING_STEPS)

CARGO := $(or $(CARGO),cargo)

all: buildsteps $(ALL_STEPS)

buildsteps:
	$(CARGO) build --release --bins

$(DEBUG_STEPS): .FORCE
	$(CARGO) build --bin $@
	cp target/debug/$@$(EXEC_EXT) ./$@

$(STEPS): .FORCE
	$(CARGO) build --release --bin $@
	cp target/release/$@$(EXEC_EXT) ./$@

$(REPEATING_STEPS): $(lastword $(STEPS))
	cp $^ ./$@

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
	rm -f $(ALL_STEPS)
