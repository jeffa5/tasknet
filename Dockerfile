FROM rust as builder

WORKDIR /app

RUN cargo init --bin tasknet-server && \
    cargo init --lib tasknet-web && \
    cargo init --lib tasknet-sync && \
    cargo init --lib kratos-api

COPY Cargo.lock Cargo.toml ./
COPY tasknet-server/Cargo.toml ./tasknet-server/Cargo.toml
COPY tasknet-web/Cargo.toml ./tasknet-web/Cargo.toml
COPY tasknet-sync/Cargo.toml ./tasknet-sync/Cargo.toml
COPY kratos-api/Cargo.toml ./kratos-api/Cargo.toml

FROM builder as web

RUN curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
RUN cd tasknet-web && wasm-pack build --target web --release

COPY tasknet-web tasknet-web
COPY tasknet-sync tasknet-sync
COPY kratos-api kratos-api

RUN touch tasknet-server/src/main.rs && touch tasknet-sync/src/lib.rs && touch kratos-api/src/lib.rs && \
    cd tasknet-web && \
    wasm-pack build --target web --out-name package --release && \
    mkdir -p /out/tasknet/pkg && \
    cp pkg/package_bg.wasm /out/tasknet/pkg/. && \
    cp pkg/package.js /out/tasknet/pkg/. && \
    cp -r assets /out/tasknet/. && \
    cp -r styles /out/tasknet/. && \
    cp index.html /out/tasknet/. && \
    cp service-worker.js /out/tasknet/. && \
    cp tasknet.webmanifest /out/tasknet/.

FROM builder as server

COPY --from=builder /app /app

RUN cargo build --release

COPY tasknet-server tasknet-server
COPY tasknet-sync tasknet-sync
COPY kratos-api kratos-api

RUN touch tasknet-server/src/main.rs && touch tasknet-sync/src/lib.rs && touch kratos-api/src/lib.rs && cargo build --release

FROM debian

COPY --from=web /out /out
COPY --from=server /app/target/release/tasknet-server /tasknet-server

ENTRYPOINT ["/tasknet-server", "--static-files-dir", "/out"]
