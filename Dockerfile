FROM rust as builder
RUN apt-get update && apt-get -y install golang clang && rm -rf /var/lib/apt/lists/*
WORKDIR /src
COPY ./Cargo.toml ./
COPY ./Cargo.lock ./
COPY ./crates ./crates
RUN cargo build --release

FROM gcr.io/distroless/cc-debian12
COPY --from=builder /src/target/release/grustonnet-ls /bin/grustonnet-ls
COPY --from=builder /src/target/release/grustonnet-lint /bin/grustonnet-lint
CMD ["grustonnet-ls"]

