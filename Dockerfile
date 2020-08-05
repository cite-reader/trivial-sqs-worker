FROM rust:1.45.2-slim-stretch
RUN mkdir src
WORKDIR src
COPY . .
RUN set -eux; \
	apt-get update; \
	apt-get install -y --no-install-recommends pkg-config libssl-dev
ENV RUSTFLAGS -C force-frame-pointers=yes
RUN ["cargo", "build", "--release"]

FROM debian:stretch-slim
RUN set -eux; \
	apt-get update; \
	apt-get install -y --no-install-recommends ca-certificates
COPY --from=0 /src/target/release/trivial-sqs-worker /usr/local/bin/trivial-sqs-worker
ENTRYPOINT ["trivial-sqs-worker"]
