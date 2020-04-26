build: target/release/ldpl-rs
	cp target/release/ldpl-rs .

target/release/ldpl-rs: src/*.rs
	cargo build --release

test:
	cargo test

ldpltest: test build
	cp ldpl-rs ldpltest/
	cd ldpltest/ && sh compileAndRunTester.sh