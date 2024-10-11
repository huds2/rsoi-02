FROM rust:buster as build

COPY ./flights ./flights
COPY ./requester ./requester
COPY ./structs ./structs
WORKDIR ./flights

RUN cargo build --release

FROM debian:buster
COPY --from=build /flights/target/release/flights .
CMD ["./flights"]
