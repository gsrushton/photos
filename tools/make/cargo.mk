
ROOT_CARGO_DIR := $(ROOT_BUILD_DIR)/target

define cargo-build-dir
$(ROOT_CARGO_DIR)/$(if $2,$2/,)$1
endef

define build-cargo-project-template
ARGS := --package $1

ifeq ("$4", "release")
	ARGS := $$(ARGS) --release
endif

ifneq ("$5", "")
	ARGS := $$(ARGS) --target $5
endif

ifneq ("$6", "")
	ARGS := $$(ARGS) $6
endif

TARGET_DIR := $(call cargo-build-dir,$4,$5)
TARGET_STEM := $$(TARGET_DIR)/$(if $2,$2,$1)
TARGET := $$(TARGET_STEM)$3

$$(TARGET): TARGET_STEM := $$(TARGET_STEM)
$$(TARGET): ARGS := $$(ARGS)
$$(TARGET): | $(ROOT_BUILD_DIR)
	cargo build $$(ARGS) --target-dir $(ROOT_CARGO_DIR)
	@bin/dpp $(CURDIR) $$(TARGET_STEM).d $$(TARGET_STEM).pp.d.temp
	@mv $$(TARGET_STEM).pp.d.temp $$(TARGET_STEM).pp.d

.PRECIOUS: $$(TARGET_STEM).pp.d
$$(TARGET_STEM).pp.d: ;
include $$(wildcard $$(TARGET_STEM).pp.d)
endef

define build-cargo-project
$(strip $(eval $(call build-cargo-project-template,$1,$2,$3,$4,$5,$6)) $(TARGET))
endef

define build-cargo-bin
$(strip $(eval $(call build-cargo-project-template,$1,,,$2,$3,$4)) $(TARGET))
endef

define build-cargo-wasm-project-template
PACKAGE := $(subst -,_,$1)

TARGET := $$(call build-cargo-project,$1,$$(PACKAGE),.wasm,$2,wasm32-unknown-unknown)

TARGET_JS := $$(TARGET_DIR)/$1.js
$$(TARGET_JS): TARGET_DIR := $$(TARGET_DIR)
$$(TARGET_JS): $$(TARGET)
	wasm-bindgen --target web --out-dir $$(TARGET_DIR) --out-name $1 $$^

TARGET_WASM := $$(TARGET_DIR)/$1_bg.wasm
$$(TARGET_WASM): $$(TARGET_JS)

TARGETS := $$(TARGET_JS) $$(TARGET_WASM)
endef

define build-cargo-wasm-project
$(strip $(eval $(call build-cargo-wasm-project-template,$1,$2,wasm32-unknown-unknown)) $(TARGETS))
endef
