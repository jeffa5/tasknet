.PHONY: web
web:
	cd web && trunk build

.PHONY: server
server:
	cargo build -p tasknet-server

.PHONY: run
run: web server
	cargo run -p tasknet-server