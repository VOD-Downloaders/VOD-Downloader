###############################################################################
# Install rust dependencies once (reused)
###############################################################################
FROM rust:alpine AS chef

RUN apk add --no-cache \
        build-base \
        ca-certificates \
        curl \
        musl-dev \
        protoc \
        protobuf-dev \
    && cargo install cargo-chef

WORKDIR /build
 
###############################################################################
# Compute the dependency fingerprint from Cargo.toml + Cargo.lock (only reruns when those files change)
###############################################################################
FROM chef AS planner
 
# Only needs a minimal main instead of all source
RUN mkdir src && echo 'fn main() {}' > src/main.rs
COPY Cargo.lock .
COPY Cargo.toml .

RUN cargo chef prepare --recipe-path recipe.json
 
###############################################################################
# Build the dependencies when Cargo.toml + Cargo.lock change
###############################################################################
FROM chef AS rust-builder

COPY --from=planner /build/recipe.json recipe.json

RUN cargo chef cook --release --recipe-path recipe.json

###############################################################################
# Compile actual fmhy_downloader
###############################################################################
COPY src/ ./src
COPY Cargo.lock .
COPY Cargo.toml .

RUN cargo build --release

###############################################################################
# Actual runtime container
###############################################################################
FROM alpine:latest

ARG PUID=1000
ARG PGID=1000
ARG TZ=UTC

ARG CHOWN_CONFIG=true
ARG CHOWN_OUTPUT=true

ARG APP_BIN=fmhy_downloader
ARG APP_USER=fmhydownloader
ARG WEBUI_PORT=8080

ENV PUID=${PUID} \
    PGID=${PGID} \
    TZ=${TZ} \
	CHOWN_CONFIG=${CHOWN_CONFIG} \
	CHOWN_OUTPUT=${CHOWN_OUTPUT} \
    APP_USER=${APP_USER} \
    APP_BIN=${APP_BIN} \
    WEBUI_PORT=${WEBUI_PORT}

WORKDIR /app

# Utilities. shadow provides groupadd/useradd, su-exec is alpine's gosu
RUN apk add --no-cache \
        bash \
        ca-certificates \
        curl \
        procps \
        shadow \
        su-exec \
        tzdata

# Copy the compiled Rust binary from the build stage
COPY --from=rust-builder /build/target/release/${APP_BIN} /app/${APP_BIN}
RUN chmod +x /app/${APP_BIN} 

# Copy web files
COPY web/src/ ./web

# Copy entrypoint
COPY entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

# Create a config directory and make it the cwd
VOLUME /config

EXPOSE ${WEBUI_PORT}

# Pass only when the body reports "healthy" 
HEALTHCHECK --interval=20s --timeout=5s --start-period=10s --retries=3 \
	CMD curl -s "http://localhost:${WEBUI_PORT}/health" | grep -q '"health":"healthy"' || exit 1

ENTRYPOINT ["/entrypoint.sh"]
CMD ["/app/fmhy_downloader"]
