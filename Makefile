CERTS_DIR ?= certs
CA_KEYS := $(CERTS_DIR)/ca.pem $(CERTS_DIR)/ca-key.pem $(CERTS_DIR)/ca.csr
SERVER_KEYS := $(CERTS_DIR)/server.crt $(CERTS_DIR)/server.key $(CERTS_DIR)/server.csr

$(CA_KEYS): $(CERTS_DIR)/ca-csr.json
	cfssl gencert -initca $(CERTS_DIR)/ca-csr.json | cfssljson -bare $(CERTS_DIR)/ca -

$(SERVER_KEYS): $(CA_KEYS) $(CERTS_DIR)/ca-config.json $(CERTS_DIR)/server.json
	cfssl gencert -ca=$(CERTS_DIR)/ca.pem -ca-key=$(CERTS_DIR)/ca-key.pem -config=$(CERTS_DIR)/ca-config.json -profile=server $(CERTS_DIR)/server.json | cfssljson -bare $(CERTS_DIR)/server -
	mv $(CERTS_DIR)/server.pem $(CERTS_DIR)/server.crt
	mv $(CERTS_DIR)/server-key.pem $(CERTS_DIR)/server.key

.PHONY: serve
serve: $(SERVER_KEYS) web
	RUST_LOG=info cargo run --bin tasknet-server

.PHONY: web
web: web-build web-pkg web-statics

.PHONY: web-pkg
web-pkg:
	mkdir -p tasknet-web/local/tasknet/pkg
	cp tasknet-web/pkg/package_bg.wasm tasknet-web/local/tasknet/pkg/.
	cp tasknet-web/pkg/package.js tasknet-web/local/tasknet/pkg/.

.PHONY: web-statics
web-statics:
	cp -r tasknet-web/assets tasknet-web/local/tasknet/.
	cp -r tasknet-web/styles tasknet-web/local/tasknet/.
	cp tasknet-web/index.html tasknet-web/local/tasknet/.
	cp tasknet-web/service-worker.js tasknet-web/local/tasknet/.
	cp tasknet-web/tasknet.webmanifest tasknet-web/local/tasknet/.

.PHONY: web-build
web-build: web-clean
	cd tasknet-web && wasm-pack build --target web --out-name package --release

.PHONY: web-clean
web-clean:
	rm -rf tasknet-web/local

.PHONY: web-test
web-test: web-test-firefox web-test-chrome web-test-safari

.PHONY: web-test-safari
web-test-safari:
	cd tasknet-web && wasm-pack test --headless --safari

.PHONY: web-test-chrome
web-test-chrome:
	cd tasknet-web && wasm-pack test --headless --chrome

.PHONY: web-test-firefox
web-test-firefox:
	cd tasknet-web && wasm-pack test --headless --firefox

.PHONY: clean
clean: web-clean
	rm -f $(CA_KEYS) $(SERVER_KEYS)
	cargo clean
