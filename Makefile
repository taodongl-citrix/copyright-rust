
target := target/release/webhook target/release/work
.PHONY: all

all: $(target) deploy

$(target): $(shell find webhook work -type f -name '*.rs')
	cargo build --release
deploy: build/work build/webhook build/askpass.sh build/run.sh

# target/release/webhook: $(shell find webhook -type f -name '*.rs')

# target/release/work: $(shell find work -type f -name '*.rs')

build/work: target/release/work
	@mkdir -p build
	cp $< $@
build/webhook: target/release/webhook
	@mkdir -p build
	cp $< $@
build/askpass.sh: askpass.sh
	@mkdir -p build
	cp $< $@
build/run.sh: run.sh
	@mkdir -p build
	cp $< $@

