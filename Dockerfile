FROM rust:1.75 as builder

WORKDIR /usr/src/app
COPY . .

RUN cargo install --path .

FROM debian:stable
RUN apt-get update && apt-get upgrade
COPY --from=builder /usr/local/cargo/bin/ /usr/local/bin/

EXPOSE 80

CMD ["zeus-controller"]
