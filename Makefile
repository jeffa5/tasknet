CERTS_DIR ?= certs
CA_KEYS := $(CERTS_DIR)/ca.pem $(CERTS_DIR)/ca-key.pem $(CERTS_DIR)/ca.csr
SERVER_KEYS := $(CERTS_DIR)/server.crt $(CERTS_DIR)/server.key $(CERTS_DIR)/server.csr

TASKNET_WEB_LOCAL := tasknet-web/local
TASKNET_WEB_LOCAL_BUILD := tasknet-web/local-build

$(CA_KEYS): $(CERTS_DIR)/ca-csr.json
	cfssl gencert -initca $(CERTS_DIR)/ca-csr.json | cfssljson -bare $(CERTS_DIR)/ca -

$(SERVER_KEYS): $(CA_KEYS) $(CERTS_DIR)/ca-config.json $(CERTS_DIR)/server.json
	cfssl gencert -ca=$(CERTS_DIR)/ca.pem -ca-key=$(CERTS_DIR)/ca-key.pem -config=$(CERTS_DIR)/ca-config.json -profile=server $(CERTS_DIR)/server.json | cfssljson -bare $(CERTS_DIR)/server -
	mv $(CERTS_DIR)/server.pem $(CERTS_DIR)/server.crt
	mv $(CERTS_DIR)/server-key.pem $(CERTS_DIR)/server.key

.PHONY: serve
serve: $(SERVER_KEYS) web docker-build docker-run

.PHONY: migrate
migrate:
	flyway migrate

.PHONY: web
web: web-build web-pkg web-statics
	rm -rf $(TASKNET_WEB_LOCAL)
	mv $(TASKNET_WEB_LOCAL_BUILD) $(TASKNET_WEB_LOCAL)

.PHONY: web-pkg
web-pkg:
	mkdir -p $(TASKNET_WEB_LOCAL_BUILD)/tasknet/pkg
	cp tasknet-web/pkg/package_bg.wasm $(TASKNET_WEB_LOCAL_BUILD)/tasknet/pkg/.
	cp tasknet-web/pkg/package.js $(TASKNET_WEB_LOCAL_BUILD)/tasknet/pkg/.

.PHONY: web-statics
web-statics:
	cp -r tasknet-web/assets $(TASKNET_WEB_LOCAL_BUILD)/tasknet/.
	cp -r tasknet-web/styles $(TASKNET_WEB_LOCAL_BUILD)/tasknet/.
	cp tasknet-web/index.html $(TASKNET_WEB_LOCAL_BUILD)/tasknet/.
	cp tasknet-web/service-worker.js $(TASKNET_WEB_LOCAL_BUILD)/tasknet/.
	cp tasknet-web/tasknet.webmanifest $(TASKNET_WEB_LOCAL_BUILD)/tasknet/.

.PHONY: web-build
web-build:
	rm -rf $(TASKNET_WEB_LOCAL_BUILD)
	cd tasknet-web && wasm-pack build --target web --out-name package --release

.PHONY: web-clean
web-clean:
	rm -rf $(TASKNET_WEB_LOCAL) $(TASKNET_WEB_LOCAL_BUILD)

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
clean: web-clean db-clean
	rm -f $(CA_KEYS) $(SERVER_KEYS)
	cargo clean

.PHONY: docker-build
docker-build:
	nix build .#docker-server
	docker load -i result

.PHONY: docker-run
docker-run:
	docker-compose up

.PHONY: db-clean
db-clean:
	sudo rm -rf db-data
