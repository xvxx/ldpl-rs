build: target/release/ldpl-rs
	cp target/release/ldpl-rs .

target/release/ldpl-rs: src/*.rs
	cargo build --release
