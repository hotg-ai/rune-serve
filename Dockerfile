FROM rust:1.51.0 as builder

WORKDIR /app

RUN rustup self update
RUN rustup update
RUN rustup toolchain install stable

COPY . /app
RUN cargo fetch


RUN cargo +stable build --release

RUN set -x \
	&& DEBIAN_FRONTEND=noninteractive apt-get update -qq \
	&& DEBIAN_FRONTEND=noninteractive apt-get install -qq -y --no-install-recommends --no-install-suggests \
	curl \
	ca-certificates \
	unzip \
	&& DEBIAN_FRONTEND=noninteractive apt-get purge -qq -y --auto-remove -o APT::AutoRemove::RecommendsImportant=false apt-utils \
	&& rm -rf /var/lib/apt/lists/* \
	\
	&& URL="$( curl -sS  https://ngrok.com/download | grep -Eo "href=\".+ngrok-stable-linux-amd64\.zip\"" | awk -F'\"' '{print $2}' )" \
	&& curl -sS -o /tmp/ngrok.zip ${URL} \
	&& unzip /tmp/ngrok.zip \
	&& rm /tmp/ngrok.zip \
	&& mv ngrok /usr/local/bin \
	&& ngrok version | grep -E '^ngrok.+[.0-9]+$' \
	\
	&& DEBIAN_FRONTEND=noninteractive apt-get purge -qq -y --auto-remove -o APT::AutoRemove::RecommendsImportant=false \
	curl \
	ca-certificates \
	unzip \
	&& rm -rf /var/lib/apt/lists/*




FROM ubuntu:latest

WORKDIR /app

COPY --from=builder /usr/local/bin/ngrok /usr/local/bin/ngrok
COPY --from=builder /app/target/release/rune-serve /app/rune-serve

CMD ["/app/rune-serve"]