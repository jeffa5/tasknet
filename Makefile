.PHONY: web
web:
	cd web && trunk build

.PHONY: server
server:
	cargo build -p server

.PHONY: run
run:
	cargo run -p server