FROM rust:1.75.0-bullseye as builder

LABEL maintainer="ySenih@erpya.com; EdwinBetanc0urt@outlook.com;" \
	description="A Image for start service from rust binary"

WORKDIR /opt/apps/server

COPY . . /opt/apps/server/

RUN apt-get update && \
	apt-get install -y \
		pkg-config \
		libssl-dev && \
	rm -rf /var/lib/apt/lists/* && \
	cargo install --config net.git-fetch-with-cli=true --path . && \
	mv docker/template.env /usr/local/cargo/bin/ && \
	mv docker/start.sh /usr/local/cargo/bin/



FROM debian:bullseye

ENV \
	RUST_LOG="info" \
	S3_URL="-" \
	BUCKET_NAME="-" \
	API_KEY="-" \
	SECRET_KEY="-" \
	ALLOWED_ORIGIN="*" \
	TZ="America/Caracas"

COPY --from=builder /usr/local/cargo/bin/server /usr/local/bin/server

WORKDIR /opt/apps/server

COPY --from=builder /usr/local/cargo/bin/template.env /opt/apps/server/template.env
COPY --from=builder /usr/local/cargo/bin/start.sh /opt/apps/server/start.sh

RUN apt-get update && \ 
	apt-get install -y \
		pkg-config \
		openssl \
		libssl-dev \
		tzdata && \
	rm -rf /var/lib/apt/lists/*

RUN addgroup adempiere && \
	adduser --disabled-password --gecos "" --ingroup adempiere --no-create-home adempiere && \
	chown -R adempiere /opt/apps/server/ && \
	chmod +x /usr/local/bin/server && \
	echo "Set Timezone..." && \
	echo $TZ > /etc/timezone

USER adempiere


# Start app
ENTRYPOINT ["sh" , "start.sh"]
