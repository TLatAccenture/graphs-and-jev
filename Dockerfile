# syntax=docker/dockerfile:1
FROM rust:1.97-bookworm@sha256:0e2bcaef56d041a486784e54104a81aebe0da44bd03019bd70bc0401e42e4a97 AS build
RUN sed -i 's#http://deb.debian.org#https://deb.debian.org#g' /etc/apt/sources.list.d/debian.sources \
    && apt-get update \
    && apt-get install -y --no-install-recommends musl-tools \
    && rm -rf /var/lib/apt/lists/* \
    && rustup target add x86_64-unknown-linux-musl
WORKDIR /build
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --locked --release --target x86_64-unknown-linux-musl \
    && file target/x86_64-unknown-linux-musl/release/serve | grep -Eq 'ELF 64-bit.*x86-64' \
    && ! readelf -l target/x86_64-unknown-linux-musl/release/serve | grep -q INTERP \
    && ! readelf -d target/x86_64-unknown-linux-musl/release/serve | grep -q NEEDED

FROM scratch
WORKDIR /app
COPY --from=build --chown=65532:65532 /build/target/x86_64-unknown-linux-musl/release/serve /app/serve
COPY --chown=65532:65532 static /app/static
COPY --chown=65532:65532 catalogue.json /app/catalogue.json
ENV PORT=8080 \
    CATALOGUE_PATH=/app/catalogue.json \
    STATIC_DIR=/app/static
USER 65532:65532
EXPOSE 8080
ENTRYPOINT ["/app/serve"]
