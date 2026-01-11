# ArcBox Desktop Build Configuration
#
# Builds the macOS .app bundle with embedded daemon

CARGO := cargo
APP_NAME := ArcBox
BUNDLE_ID := com.arcbox.desktop
VERSION := 0.1.0

# Paths
BUILD_DIR := target/release
BUNDLE_DIR := target/bundle
APP_BUNDLE := $(BUNDLE_DIR)/$(APP_NAME).app
CONTENTS_DIR := $(APP_BUNDLE)/Contents
MACOS_DIR := $(CONTENTS_DIR)/MacOS
RESOURCES_DIR := $(CONTENTS_DIR)/Resources

# Binaries
DESKTOP_BIN := arcbox-desktop
DAEMON_BIN := arcbox

# Source paths (relative to this Makefile)
ARCBOX_DIR := ../arcbox

.PHONY: all clean build bundle run dev

# Default target
all: bundle

# Development build (debug)
dev:
	$(CARGO) build

# Release build
build:
	$(CARGO) build --release
	cd $(ARCBOX_DIR) && $(CARGO) build --release -p arcbox-cli

# Create the .app bundle
bundle: build
	@echo "Creating $(APP_NAME).app bundle..."
	@mkdir -p $(MACOS_DIR)
	@mkdir -p $(RESOURCES_DIR)

	# Copy binaries
	@cp $(BUILD_DIR)/$(DESKTOP_BIN) $(MACOS_DIR)/
	@cp $(ARCBOX_DIR)/target/release/$(DAEMON_BIN) $(MACOS_DIR)/

	# Copy Info.plist
	@sed -e 's/$${BUNDLE_ID}/$(BUNDLE_ID)/g' \
	     -e 's/$${APP_NAME}/$(APP_NAME)/g' \
	     -e 's/$${VERSION}/$(VERSION)/g' \
	     -e 's/$${DESKTOP_BIN}/$(DESKTOP_BIN)/g' \
	     bundle/Info.plist.template > $(CONTENTS_DIR)/Info.plist

	# Copy icon if exists
	@if [ -f bundle/AppIcon.icns ]; then \
		cp bundle/AppIcon.icns $(RESOURCES_DIR)/; \
	fi

	# Copy assets
	@if [ -d assets ]; then \
		cp -r assets $(RESOURCES_DIR)/; \
	fi

	@echo "Bundle created at $(APP_BUNDLE)"

# Run the app (development)
run: dev
	$(CARGO) run

# Run the bundled app
run-bundle: bundle
	open $(APP_BUNDLE)

# Clean build artifacts
clean:
	$(CARGO) clean
	rm -rf $(BUNDLE_DIR)

# Install to /Applications (requires sudo for system-wide)
install: bundle
	@echo "Installing $(APP_NAME).app to ~/Applications..."
	@mkdir -p ~/Applications
	@rm -rf ~/Applications/$(APP_NAME).app
	@cp -r $(APP_BUNDLE) ~/Applications/
	@echo "Installed to ~/Applications/$(APP_NAME).app"

# Create DMG for distribution
dmg: bundle
	@echo "Creating DMG..."
	@mkdir -p $(BUNDLE_DIR)/dmg
	@cp -r $(APP_BUNDLE) $(BUNDLE_DIR)/dmg/
	@ln -sf /Applications $(BUNDLE_DIR)/dmg/Applications
	@hdiutil create -volname "$(APP_NAME)" \
		-srcfolder $(BUNDLE_DIR)/dmg \
		-ov -format UDZO \
		$(BUNDLE_DIR)/$(APP_NAME)-$(VERSION).dmg
	@rm -rf $(BUNDLE_DIR)/dmg
	@echo "DMG created at $(BUNDLE_DIR)/$(APP_NAME)-$(VERSION).dmg"

# Code sign (requires valid identity)
codesign: bundle
	@echo "Code signing $(APP_NAME).app..."
	@codesign --force --deep --sign - $(APP_BUNDLE)
	@echo "Code signed (ad-hoc)"

# Show help
help:
	@echo "ArcBox Desktop Build System"
	@echo ""
	@echo "Targets:"
	@echo "  all        - Build the .app bundle (default)"
	@echo "  dev        - Development build (debug)"
	@echo "  build      - Release build"
	@echo "  bundle     - Create .app bundle"
	@echo "  run        - Run development build"
	@echo "  run-bundle - Run bundled app"
	@echo "  clean      - Clean build artifacts"
	@echo "  install    - Install to ~/Applications"
	@echo "  dmg        - Create DMG for distribution"
	@echo "  codesign   - Code sign the bundle (ad-hoc)"
	@echo "  help       - Show this help"
