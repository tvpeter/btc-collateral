DENY = -D future-incompatible -D rust_2018_idioms -D nonstandard_style

CLIPPY_DENY = -D clippy::all -D clippy::cargo -A clippy::multiple-crate-versions

init:
	git config core.hooksPath .githooks
	cd .githooks && chmod +x * && cd ..

fmt:
	cargo fmt --all -- --verbose

build:
	RUSTFLAGS="${DENY}" cargo build

build-tests:
	RUSTFLAGS="${DENY}" cargo test --no-run

test:
	RUSTFLAGS="${DENY}" RUST_BACKTRACE=1 cargo test -- --skip sudo_ --skip loop_

clippy:
	RUSTFLAGS="${DENY}" \
        cargo clippy --all-targets --all-features -- \
        ${CLIPPY_DENY}

docs:
	cargo doc --no-deps

yamllint:
	yamllint --strict .github/workflows/*.yml

.PHONY:
	audit
	build
	check-fedora-versions
	check-fedora-versions-sys
	check-typos
	clippy
	docs
	fmt
	fmt-ci
	sudo_test
	test
	test-compare-fedora-versions
	test-set-lower-bounds
	verify-dependency-bounds
	yamllint
