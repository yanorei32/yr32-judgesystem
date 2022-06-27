FROM rust:1.61.0 as builder
MAINTAINER yanorei32

WORKDIR /usr/src/judge-system
COPY . .

RUN cargo install --path .

FROM python:3.10.5-bullseye

RUN apt-get update; \
	apt-get install -y --no-install-recommends \
		sudo; \
	apt-get clean; \
	useradd -m judgeuser -u 999;

ENV USERID="judgeuser"

COPY --from=builder \
	/usr/local/cargo/bin/judge-system /usr/local/bin/judge-system

CMD ["judge-system"]
