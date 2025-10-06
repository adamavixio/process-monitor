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
	@echo "  1. Update version in src-tauri/tauri.conf.json"
	@echo "  2. git add . && git commit -m 'Bump version to vX.X.X'"
	@echo "  3. git tag vX.X.X && git push origin main --tags"
	@echo "  4. GitHub Actions will automatically build and release"

dev:
	bun run tauri dev

build:
	bun run tauri build

clean:
	rm -rf src-tauri/target
	rm -rf build
	rm -rf .svelte-kit
