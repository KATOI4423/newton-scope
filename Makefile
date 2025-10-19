
TOP_DIR:=$(shell pwd)
DOCKER_DIR:=$(TOP_DIR)/docker


# ================
# Docker commands
# ================

DOCKERIMAGE_NAME:=newton-scope-container
DOCKER_RUN_BASECMD:= \
	docker run --rm \
	-v $(TOP_DIR):/work \

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

.PHONY: backend
backend: backend-linux

.PHONY: backend-linux backend-linux-debug
backend-linux:
	$(DOCKER_RUN_BASECMD) -t $(DOCKERIMAGE_NAME) \
		"cd src-tauri && cargo build --release --target x86_64-unknown-linux-gnu"

backend-linux-debug:
	$(DOCKER_RUN_BASECMD) -t $(DOCKERIMAGE_NAME) \
		"cd src-tauri && cargo build --target x86_64-unknown-linux-gnu"
