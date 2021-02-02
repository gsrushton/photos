ROOT_BUILD_DIR := build

.PHONY: all
all:

.PHONY: debug
debug:

.PHONY: release
release:

$(ROOT_BUILD_DIR):
	@mkdir -p $@

include tools/make/cargo.mk

define dirname
$(patsubst %/,%,$(dir $1))
endef

define copy-dir-template
DST_FILES := $(patsubst $1/%,$2/%,$(shell find $1 -mindepth 1 -maxdepth 1 -type f)) \
             $(foreach C,$(shell find $1 -mindepth 1 -maxdepth 1 -type d),$(call copy-dir,$C,$2/$(patsubst $1/%,%,$C)))

$2: | $(call dirname,$2)
	@mkdir $$@

$2/%: $1/% | $2
	cp $$^ $$@
endef

define copy-dir
$(strip $(eval $(call copy-dir-template,$1,$2)) $(DST_FILES))
endef

WEB_CLIENT_TARGETS := $(call build-cargo-wasm-project,photos-web-client,release)

DOCKER_DIR := $(ROOT_BUILD_DIR)/docker
$(DOCKER_DIR): | $(ROOT_BUILD_DIR)
	@mkdir $@

DKR_PHOTOSD_DIR := $(DOCKER_DIR)/photosd
$(DKR_PHOTOSD_DIR): | $(DOCKER_DIR)
	@mkdir $@

$(DKR_PHOTOSD_DIR)/Dockerfile: tools/docker/photosd/Dockerfile | $(DKR_PHOTOSD_DIR)
	cp $^ $@

$(DKR_PHOTOSD_DIR)/photosd: $(call build-cargo-bin,photosd,release)
	cp $^ $@

DKR_PHOTOSD_SCRIPT_DIR := $(DKR_PHOTOSD_DIR)/share/www/script
$(DKR_PHOTOSD_SCRIPT_DIR): | $(DKR_PHOTOSD_DIR)
	@mkdir $@

CARGO_WASM_RELEASE_DIR := $(call cargo-build-dir,release,wasm32-unknown-unknown)
$(DKR_PHOTOSD_SCRIPT_DIR)/%: $(CARGO_WASM_RELEASE_DIR)/% | $(DKR_PHOTOSD_SCRIPT_DIR)
	cp -r $^ $@

$(CARGO_WASM_RELEASE_DIR)/snippets: $(WEB_CLIENT_TARGETS)

$(DOCKER_DIR)/photosd-prepare: $(DKR_PHOTOSD_DIR)/Dockerfile \
                               $(DKR_PHOTOSD_DIR)/photosd \
                               $(call copy-dir,crates/web-server/share,$(DKR_PHOTOSD_DIR)/share) \
                               $(DKR_PHOTOSD_SCRIPT_DIR)/snippets \
                               $(subst $(CARGO_WASM_RELEASE_DIR),$(DKR_PHOTOSD_SCRIPT_DIR),$(WEB_CLIENT_TARGETS))
	@touch $@

$(DOCKER_DIR)/photosd-build: $(DOCKER_DIR)/photosd-prepare
	docker build -t photosd:latest $(DKR_PHOTOSD_DIR)
	@touch $@

$(DOCKER_DIR)/photosd-publish: $(DOCKER_DIR)/photosd-build
	docker tag photosd:latest odin:5000/photosd:latest
	docker push odin:5000/photosd:latest

DKR_PHOTOS_DIR := $(DOCKER_DIR)/photos
$(DKR_PHOTOS_DIR): | $(DOCKER_DIR)
	@mkdir $@

$(DKR_PHOTOS_DIR)/Dockerfile: tools/docker/photos/Dockerfile | $(DKR_PHOTOS_DIR)
	cp $^ $@

$(DKR_PHOTOS_DIR)/photos: $(call build-cargo-bin,photos,release)
	cp $^ $@

$(DOCKER_DIR)/photos-build: $(DKR_PHOTOS_DIR)/Dockerfile \
		                    $(DKR_PHOTOS_DIR)/photos
	docker build -t photos:latest $(DKR_PHOTOS_DIR)
	@touch $@

$(DOCKER_DIR)/photos-publish: $(DOCKER_DIR)/photos-build
	docker tag photos:latest odin:5000/photos:latest
	docker push odin:5000/photos:latest
