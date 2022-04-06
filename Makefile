TAG := $(shell git describe --tags `git rev-list --tags --max-count=1`)

check/tag:
	@echo "TAG: $(TAG)"
	@echo -n "Are you sure? [y/N] " && read ans && [ $${ans:-N} = y ]

test:
	cargo test

build:
	cargo build --release

run:
	cargo run

# Podman clean
clean-podman:
	podman image prune
	podman image rm miniq
	podman image rm miniq-alpine


# Podman Build
build-podman:
	podman build --platform linux/amd64,linux/arm64 --format docker --manifest miniq .

push-podman: check/tag
	podman login docker.io
	podman manifest push --format v2s2 miniq docker://docker.io/ac5tin/miniq:${TAG}



# Podman build (Alpine)
build-podman/alpine:
	podman build --platform linux/amd64,linux/arm64 --format docker --manifest miniq-alpine . -f Dockerfile.alpine

push-podman/alpine: check/tag
	podman login docker.io
	podman manifest push --format v2s2 miniq-alpine docker://docker.io/ac5tin/miniq:${TAG}-alpine