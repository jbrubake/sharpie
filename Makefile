UI = ./ui/app-window.slint
STYLE ?= fluent

gui:
	cargo run

live-preview:
	SLINT_LIVE_PREVIEW=1 cargo run --features slint/live-preview

build:
	cargo build

preview:
	slint-viewer --style $(STYLE) $(UI)

docs:
	cargo doc

view-docs:
	cargo doc --open

