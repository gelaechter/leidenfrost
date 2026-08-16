.PHONY: help run verbose debug profile

# Default target
help:
	@echo "Different run configurations:"
	@echo "  make run     	- Run leidenfrost in release mode"
	@echo "  make verbose   - Run leidenfrost in verbose release mode"
	@echo "  make debug   	- Run leidenfrost with all debug flags"
	@echo "  make profile 	- Build leidenfrost and profile it with samply"

# Run in release
run:
	@cargo run --release

# Run the application with debug logging
verbose:
	@RUST_LOG="leidenfrost=debug" \
		cargo run --release

# Run the application with all debugging bells & whistles
debug:
	@RUSTFLAGS="-Awarnings --cfg tokio_unstable" \
		RUST_LOG="leidenfrost=debug" \
		cargo run -F debug --release \

# Profile the application
profile:
	@command -v samply >/dev/null 2>&1 || { \
		echo "The profile target expects you to have samply installed"; \
		echo "Install it with cargo: cargo install --locked samply"; \
		echo "or get it here: https://github.com/mstange/samply"; \
		exit 1; \
	}
	@ { [ "$$(cat /proc/sys/kernel/perf_event_paranoid 2>/dev/null)" -le -1 ]; } || { \
		echo "perf_event_paranoid must be <= -1 for samply to work"; \
		echo "(see https://github.com/mstange/samply/issues/25)"; \
		echo "Run: echo '-1' | sudo tee /proc/sys/kernel/perf_event_paranoid" \
		>&2; exit 1; \
	}
	@cargo build --profile profiling
	@samply record ./target/profiling/leidenfrost