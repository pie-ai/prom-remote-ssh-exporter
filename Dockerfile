FROM rust:1.44-buster AS build
# aarch64-linux-musl-gcc
#RUN apt update && apt install -y gcc-aarch64-linux-gnu musl-tools

# https://pixelspark.nl/2020/cross-compiling-rust-programs-for-a-raspberry-pi-from-macos
#WORKDIR /usr/libs/musl
#RUN wget https://lisa.musl.cc/9.2.1-20191012/armv7r-linux-musleabihf-cross.tgz
#RUN tar zxvf *.tgz && rm *.tgz
# pi4 not working:
# - https://lisa.musl.cc/9.2.1-20191012/aarch64-linux-musl-cross.tgz
#RUN find .
#RUN mv /usr/libs/musl/* /usr/libs/musl/linux-musl-cross

RUN cd /usr/src && USER=root cargo new --bin ssh-prometheus-exporter
WORKDIR /usr/src/ssh-prometheus-exporter
# Download the target for static linking.
#RUN rustup target add x86_64-unknown-linux-musl
#RUN rustup target add aarch64-unknown-linux-musl

# Create a dummy project and build the app's dependencies.
# If the Cargo.toml or Cargo.lock files have not changed,
# we can use the docker build cache and skip these (typically slow) steps.
#RUN USER=root cargo new app
#WORKDIR /usr/src/app
#COPY Cargo.toml Cargo.lock ./
#RUN PATH=$PATH:/usr/libs/musl/alinux-musl-cross/bin cargo build --release --target aarch64-unknown-linux-musl
#RUN pwd && find
#RUN cargo build --release

COPY dummy.rs .
COPY Cargo.toml .
# path = "src/bin/rust-prometheus-http-exporter.rs"
RUN sed -i 's#src/bin/rust-prometheus-http-exporter.rs#dummy.rs#' Cargo.toml
RUN cargo build --release
RUN sed -i 's#dummy.rs#src/bin/rust-prometheus-http-exporter.rs#' Cargo.toml
RUN find
COPY . .
RUN cargo build --release

# Copy the source and build the application.
#COPY Cargo.toml Cargo.lock ./
#COPY src ./src
#RUN cargo install --target aarch64-unknown-linux-musl --path .
#RUN cargo install --path .
RUN find .

# Copy the statically-linked binary into a scratch container.
#FROM scratch
#FROM alpine
FROM rust:1.44-buster
COPY --from=build /usr/src/ssh-prometheus-exporter/target/release/de_pa2_rust_tests .
USER 1000
CMD ["./de_pa2_rust_tests"]
