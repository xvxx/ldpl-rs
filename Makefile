
.PHONY: build
build: target/release/ldpl-rs
	cp target/release/ldpl-rs .

target/release/ldpl-rs: src/*.rs
	cargo build --release

.PHONY: clean
clean:
	cargo clean
	rm -rf ldpltest

.PHONY: test
test: ldpltest build
	cargo test
	cd ldpltest/ && sh compileAndRunTester.sh

ldpltest:
	git clone git://github.com/lartu/ldpltest
	cd ldpltest && git checkout c42160a41
	cp lib/tester.ldpl ldpltest/
