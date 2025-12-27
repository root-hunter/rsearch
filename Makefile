clear-db:
	rm -rf ./storage.db
	rm -rf ./storage.db-shm
	rm -rf ./storage.db-wal

run: clear-db
	cargo build
	RUST_LOG=info RUST_BACKTRACE=1 ./target/debug/rsearch

run-release: clear-db
	cargo build --release
	RUST_LOG=info RUST_BACKTRACE=1 ./target/release/rsearch

fmt:
	cargo fmt --all

clippy:
	cargo clippy --all -- -D warnings

audit-check:
	cargo audit

deny-warnings:
	cargo deny check

PDFIUM_ARCH=linux-x64

install-pdfium:
	rm -rf vendor/pdfium
	mkdir -p vendor/pdfium
	cd vendor/pdfium && \
	wget https://github.com/bblanchon/pdfium-binaries/releases/latest/download/pdfium-$(PDFIUM_ARCH).tgz && \
	tar -xvzf pdfium-$(PDFIUM_ARCH).tgz && \
	rm pdfium-$(PDFIUM_ARCH).tgz

install-deps: install-pdfium
	echo "All dependencies installed."