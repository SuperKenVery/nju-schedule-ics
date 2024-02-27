FROM rust:1.70.0 as builder

WORKDIR /code
COPY . repo
RUN cd repo && cargo build --release

FROM debian:trixie-slim

COPY --from=builder /code/repo/target/release/nju-schedule-ics /nju-schedule-ics

# Map config file into /config.toml

CMD [ "/nju-schedule-ics", "--config", "/config.toml" ]
