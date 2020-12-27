# Planner Stage
FROM rust:1.47 as planner
WORKDIR /app

RUN cargo install cargo-chef

COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Cacher Stage
FROM rust:1.47 as cacher
WORKDIR /app

RUN cargo install cargo-chef

COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Builder Stage
FROM rust:1.47 as builder
WORKDIR /app

# Copy over the cached dependencies
COPY --from=cacher /app/target target
COPY --from=cacher /usr/local/cargo /usr/local/cargo
COPY . .
RUN cargo build --release

# Runtime Stage
FROM debian:buster-slim as runtime
WORKDIR /app

COPY --from=builder /app/target/release/app app

COPY configuration configuration
ENV APP_ENVIRORMENT production
ENTRYPOINT ["./app"]