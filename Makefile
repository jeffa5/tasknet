.PHONY: web
web:
	cd web && trunk build

.PHONY: server
server:
	cargo build -p server

.PHONY: run
run: web server
	cargo run -p server