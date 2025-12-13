run:
	rm -rf ./storage.db
	cargo run

run-release:
	rm -rf ./storage.db
	cargo run --release