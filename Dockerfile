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


###################
# Production Base #
###################

FROM debian AS production-base
RUN apt update -y && \
    apt upgrade -y && \
    apt install -y pkg-config build-essential openssl libssl-dev


######################
# Production Package #
######################

FROM production-base AS production
COPY --from=builder /target/release/api /bin
COPY setup.sh /bin/setup.sh
COPY compose-setup.sh /bin/compose-setup.sh
CMD ["/bin/api", "server"]


###############
# CI/CD Build #
###############

FROM builder AS ci-build
RUN cargo build
RUN cargo test
