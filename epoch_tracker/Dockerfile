FROM rust:latest as builder
RUN apt-get update && apt-get -y install libpq-dev
WORKDIR /tracker
COPY . /tracker
RUN cargo build --release

FROM debian:buster-slim
COPY --from=builder ./tracker/target/release/epoch_rewards_tracker ./tracker/epoch_rewards_tracker
COPY --from=builder ./tracker/Config.toml ./tracker/Config.toml

ARG RUST_LOG
ARG ET__ENDPOINT__URL
ARG ET__STORAGE__URL
ARG ET__VALIDATOR__VOTE_ACCOUNT
ARG ET__PROMETHEUS_EXPORTER__BIND_ADDRESS