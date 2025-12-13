run:
	rm -rf ./storage.db
	cargo build
	RUST_LOG=info RUST_BACKTRACE=1 ./target/debug/rsearch

run-release:
	rm -rf ./storage.db
	cargo build --release
	RUST_LOG=info RUST_BACKTRACE=1 ./target/release/rsearch

PDFIUM_ARCH=linux-x64

install-pdfium:
	rm -rf vendor/pdfium
	mkdir -p vendor/pdfium
	cd vendor/pdfium && \
	wget https://github.com/bblanchon/pdfium-binaries/releases/download/chromium%2F7568/pdfium-$(PDFIUM_ARCH).tgz && \
	tar -xvzf pdfium-$(PDFIUM_ARCH).tgz && \
	rm pdfium-$(PDFIUM_ARCH).tgz

install-deps: install-pdfium
	echo "All dependencies installed."