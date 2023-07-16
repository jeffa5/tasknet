.PHONY: web
web:
	cd web && trunk build

.PHONY: serve
serve:
	cd web && trunk serve

.PHONY: server
server:
	cargo build -p tasknet-server

.PHONY: run
run: web server
	cargo run -p tasknet-server
