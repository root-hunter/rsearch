run:
	rm -rf ./storage.db
	cargo run

run-release:
	rm -rf ./storage.db
	cargo run --release

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