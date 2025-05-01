# Detect OS
ifeq ($(OS),Windows_NT)
	DETECTED_OS := windows
else
	UNAME_S := $(shell uname -s)
	ifeq ($(UNAME_S),Linux)
		DETECTED_OS := linux
	endif
	ifeq ($(UNAME_S),Darwin)
		DETECTED_OS := macos
	endif
endif

# Get today's date in YYYYMMDD format
TODAY := $(shell date +%Y%m%d)

# Define upload directory
UPLOAD_DIR := ../upload

all: build

build:
	cargo build

release:
	cargo build --release

static:
	cargo build --release
	@echo "Static library is available at target/release/libversedb.a"

dynamic:
	cargo build --release
	@echo "Dynamic library is available at target/release/libversedb.so"

test:
	cargo test

wasm32:
	cargo build --target wasm32-unknown-unknown --features wasm

testwasm32:
	wasm-pack test --chrome --headless --features wasm

# Package native build
package-native: release
	@mkdir -p dist
	@cp target/release/libversedb.* dist/
	@zip -r versedb_$(DETECTED_OS)_$(TODAY).zip dist/
	@rm -rf dist
	@echo "Native package created: versedb_$(DETECTED_OS)_$(TODAY).zip"

# Package wasm build
package-wasm: wasm32
	@mkdir -p dist
	@cp target/wasm32-unknown-unknown/debug/versedb.wasm dist/
	@zip -r versedb_wasm32_$(TODAY).zip dist/
	@rm -rf dist
	@echo "WASM package created: versedb_wasm32_$(TODAY).zip"

# Package both native and wasm
package: package-native package-wasm
	@mkdir -p $(UPLOAD_DIR)
	@mv versedb_*.zip $(UPLOAD_DIR)/
	@echo "All packages moved to $(UPLOAD_DIR)/"
	@echo "Available packages in $(UPLOAD_DIR):"
	@ls -l $(UPLOAD_DIR)/versedb_*.zip

clean:
	cargo clean
	rm -f versedb_*.zip
	rm -rf dist
	rm -f $(UPLOAD_DIR)/versedb_*.zip

.PHONY: all build release static dynamic test wasm32 testwasm32 package package-native package-wasm clean
