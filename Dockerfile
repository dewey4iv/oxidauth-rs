###########
# Planner #
###########

# FROM registry.vizerapp.dev/lib/rust AS planner
FROM rust AS planner
WORKDIR /app
RUN cargo install cargo-chef
COPY . .
RUN cargo chef prepare --recipe-path recipe.json


##########
# Cacher #
##########

FROM rust AS cacher
WORKDIR /app
RUN cargo install cargo-chef
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --target-dir /target --release --recipe-path recipe.json
RUN cargo build --release --target-dir /target --bin api

###########
# Builder #
###########

# FROM registry.vizerapp.dev/lib/oxidauth-cacher AS builder
FROM cacher AS builder
WORKDIR /app
COPY . .
RUN cargo build --release --target-dir /target --bin api


######################
# Production Package #
######################

FROM debian AS production
COPY --from=builder /target/release/api /bin
RUN mkdir uploads
CMD ["/bin/api"]


###############
# CI/CD Build #
###############

FROM builder AS ci-build
RUN cargo build
RUN cargo test