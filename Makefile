
TOP_DIR:=$(shell pwd)
DOCKER_DIR:=$(TOP_DIR)/docker
CARGO_CACHE_DIR:=$(TOP_DIR)/.cargo/registry

.PHONY: all clean distclean
all: tauri
clean: tauri-clean
distclean: tauri-distclean

# ================
# Docker commands
# ================

DOCKERIMAGE_NAME:=newton-scope-container
DOCKER_RUN_BASECMD:= \
	docker run --rm \
	-u $(shell id -u):$(shell id -g) \
	-v $(TOP_DIR):/work \
 	-v $(CARGO_CACHE_DIR):/home/ubuntu/.cargo/registry

.PHONY: image container-run
image:
	@docker image inspect $(DOCKERIMAGE_NAME) > /dev/null 2>&1 \
		&& docker rmi -f $(DOCKERIMAGE_NAME) || true
	@docker build \
		--build-arg USER_UID=$(shell id -u) \
		--build-arg USER_GID=$(shell id -g) \
		-t $(DOCKERIMAGE_NAME) $(DOCKER_DIR)
		
container-run:
	$(DOCKER_RUN_BASECMD) -it $(DOCKERIMAGE_NAME) \
		"bash -i"

# ================
# Build commands
# ================

.PHONY: tauri
tauri: tauri-linux

.PHONY: tauri-linux tauri-linux-debug
tauri-linux: cargo-cache
	$(DOCKER_RUN_BASECMD) -t $(DOCKERIMAGE_NAME) \
		"cd src-tauri && cargo build --release --target x86_64-unknown-linux-gnu"

tauri-linux-debug: cargo-cache
	$(DOCKER_RUN_BASECMD) -t $(DOCKERIMAGE_NAME) \
		"cd src-tauri && cargo build --target x86_64-unknown-linux-gnu"

cargo-cache: .cargo
.cargo:
	@-mkdir -p $(CARGO_CACHE_DIR)


.PHONY: tauri-clean tauri-distclean
tauri-clean:
	$(DOCKER_RUN_BASECMD) -t $(DOCKERIMAGE_NAME) \
		"cd src-tauri && cargo clean"

tauri-distclean: tauri-clean
	rm -rf $(CARGO_CACHE_DIR)
