.PHONY: serve
serve: web
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
	cargo clean
