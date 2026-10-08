# syntax=docker/dockerfile:1
FROM rust:1.97-bookworm@sha256:0e2bcaef56d041a486784e54104a81aebe0da44bd03019bd70bc0401e42e4a97 AS build
WORKDIR /build
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --locked --release

FROM gcr.io/distroless/cc-debian12:nonroot@sha256:777e96cf322c46bc32aca926c263624c4dc8d7cf37e2fa65ba2c7e697318ebbb
WORKDIR /app
COPY --from=build --chown=65532:65532 /build/target/release/serve /app/serve
COPY --chown=65532:65532 static /app/static
COPY --chown=65532:65532 catalogue.json /app/catalogue.json
ENV PORT=8080 \
    CATALOGUE_PATH=/app/catalogue.json \
    STATIC_DIR=/app/static
USER 65532:65532
EXPOSE 8080
ENTRYPOINT ["/app/serve"]
