FROM rust:1-slim-bullseye as chef
RUN cargo install cargo-chef 
WORKDIR /app
RUN apt update && apt install lld clang -y

FROM chef as planner
COPY . .
RUN cargo chef prepare  --recipe-path recipe.json

FROM chef as builder
COPY --from=planner /app/recipe.json recipe.json
RUN apt-get install -y libssl-dev pkg-config protobuf-compiler
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
ENV SQLX_OFFLINE true
RUN cargo build --release --bin rinha

FROM debian:bullseye-slim AS runtime
WORKDIR /app
COPY --from=builder /app/target/release/rinha rinha
RUN apt-get update
RUN apt-get install -y ca-certificates
ENV APP_ENVIRONMENT production
ENTRYPOINT ["./rinha"]
