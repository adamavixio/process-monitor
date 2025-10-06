.PHONY: help dev build clean

help:
	@echo "Process Monitor - Build Commands"
	@echo ""
	@echo "Available commands:"
	@echo "  make dev    - Run development server"
	@echo "  make build  - Build production app locally"
	@echo "  make clean  - Clean build artifacts"
	@echo ""
	@echo "To create a release:"
	@echo "  1. Update version in src-tauri/tauri.conf.json (e.g., 0.1.6)"
	@echo "  2. git add . && git commit -m 'Bump version to v0.1.6'"
	@echo "  3. git tag v0.1.6"
	@echo "  4. git push origin main --tags"
	@echo "  5. GitHub Actions will build, sign, notarize, and release automatically"

dev:
	bun run tauri dev

build:
	bun run tauri build

clean:
	rm -rf src-tauri/target
	rm -rf build
	rm -rf .svelte-kit
