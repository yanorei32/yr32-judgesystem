FROM rust:1.62.0 as builder
MAINTAINER yanorei32

WORKDIR /usr/src/judge-system
COPY . .

RUN cargo install cargo-credits; \
	cargo credits; \
	mkdir -p /usr/share/licenses/judge-system; \
	cp LICENSE CREDITS /usr/share/licenses/judge-system/; \
	cargo install --path .

FROM python:3.10.5-bullseye

RUN apt-get update; \
	apt-get install -y --no-install-recommends \
		sudo; \
	apt-get clean; \
	useradd -m judgeuser -u 999;

ENV USERID="judgeuser"

COPY --from=builder \
	/usr/share/licenses/judge-system /usr/share/licenses/judge-system

COPY --from=builder \
	/usr/local/cargo/bin/judge-system /usr/local/bin/judge-system

CMD ["judge-system"]
