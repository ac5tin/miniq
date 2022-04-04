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

clean-podman:
	podman image prune
	podman image rm miniq

build-podman:
	podman build --platform linux/amd64,linux/arm64 --format docker --manifest miniq .

push-podman: check/tag
	podman login docker.io
	podman manifest push --format v2s2 miniq docker://docker.io/ac5tin/miniq:${TAG}