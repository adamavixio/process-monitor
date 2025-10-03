.PHONY: help dev build release clean

help:
	@echo "Process Monitor - Build Commands"
	@echo ""
	@echo "Available commands:"
	@echo "  make dev       - Run development server"
	@echo "  make build     - Build production app"
	@echo "  make release   - Build and create GitHub release"
	@echo "  make clean     - Clean build artifacts"

dev:
	bun run tauri dev

build:
	APPLE_API_KEY=JKZV3THTDC \
	APPLE_API_ISSUER=b4435452-af0c-44e8-87b7-836f506db990 \
	APPLE_API_KEY_PATH=~/.tauri-keys/AuthKey_JKZV3THTDC.p8 \
	bun run tauri build

release: build
	@echo "Creating GitHub release..."
	@VERSION=$$(grep '"version"' src-tauri/tauri.conf.json | head -1 | sed 's/.*: "\(.*\)".*/\1/'); \
	gh release create v$$VERSION \
		--title "Process Monitor v$$VERSION" \
		--notes "🎉 **Process Monitor v$$VERSION**\n\nA beautiful macOS process and port monitoring tool with a stunning cyberpunk-inspired UI.\n\n## ✨ Features\n- Real-time process and port monitoring\n- Grouped process display (process → PIDs → ports)\n- Clickable port badges to open in browser\n- Kill process functionality\n- Auto-refresh with manual controls\n- Beautiful dark theme with glowing cards\n\n## 📦 Installation\n\n1. Download the DMG file below\n2. Open the DMG and drag Process Monitor to Applications\n3. **Important:** If macOS says the app is damaged, run this command:\n   \`\`\`bash\n   xattr -cr \"/Applications/Process Monitor.app\"\n   \`\`\`\n4. Launch Process Monitor from Applications\n\n> Note: This app is not signed with an Apple Developer certificate. The command above removes the quarantine flag that macOS applies to unsigned apps.\n\n## 🤖 Built with\n- Tauri 2.1\n- Svelte 5\n- Bun\n\nGenerated with [Claude Code](https://claude.com/claude-code)" \
		"src-tauri/target/release/bundle/dmg/Process Monitor_$${VERSION}_aarch64.dmg"
	@echo "✅ Release v$$VERSION created successfully!"

clean:
	rm -rf src-tauri/target
	rm -rf build
	rm -rf .svelte-kit
