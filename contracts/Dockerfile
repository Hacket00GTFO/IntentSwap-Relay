# Stage 1: clone forge-std with a minimal git image (no root permission issues)
FROM alpine/git:latest AS git-clone
RUN git clone --depth 1 https://github.com/foundry-rs/forge-std /forge-std

# Stage 2: foundry image — copy contracts + pre-cloned lib
FROM ghcr.io/foundry-rs/foundry:latest

WORKDIR /contracts

COPY foundry.toml ./
COPY src ./src
COPY script ./script

COPY --from=git-clone /forge-std ./lib/forge-std

COPY --chmod=755 docker-entrypoint.sh /contracts/docker-entrypoint.sh

ENTRYPOINT ["/contracts/docker-entrypoint.sh"]
