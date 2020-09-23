FROM rust:1.44-buster AS build

# base utils
# https://github.com/killercup/cargo-edit
RUN cargo install cargo-edit

# create dummy project
ARG PROJECT_NAME="ssh-prometheus-exporter"
ARG BASEDIR=/var/development/src
ARG WORKDIR=${BASEDIR}/${PROJECT_NAME}

RUN mkdir -p ${BASEDIR} ; cd ${BASEDIR} && USER=root cargo new --bin ${PROJECT_NAME}
WORKDIR ${WORKDIR}

RUN cargo add ssh2@0.8
RUN cargo add csv@1.1
#RUN cargo add --features=derive serde@1.0 
RUN cargo add serde@1.0
# serde = "1.0"
# serde = { version = "1.0", features = ["derive"] }
RUN sed -i 's#serde = "1.0"#serde = { version = "1.0", features = ["derive"] }#' Cargo.toml
RUN cargo add prometheus_exporter_base@0.30.3
RUN cargo add futures@0.1.25
#RUN cargo add --features=full tokio@0.2 
RUN cargo add tokio@0.2 
# tokio = "0.2"
# tokio = { version = "0.2", features = ["full"] }
RUN sed -i 's#tokio = "0.2"#tokio = { version = "0.2", features = ["full"] }#' Cargo.toml
RUN cargo add log@0.4.8
RUN cargo add env_logger@0.7.1
#RUN cargo add clap@2.33.3
RUN cargo add pico-args@0.3.4
RUN cat Cargo.toml
RUN echo "fn main() {}" > main.rs
RUN cargo build
RUN find

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

#RUN cd /usr/src && USER=root cargo new --bin ${PROJECT_NAME}
#WORKDIR /usr/src/ssh-prometheus-exporter
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

#COPY dummy.rs .
#COPY Cargo.toml .
# path = "src/bin/rust-prometheus-http-exporter.rs"
#RUN sed -i 's#src/bin/rust-prometheus-http-exporter.rs#dummy.rs#' Cargo.toml
#RUN cargo build --release
#RUN sed -i 's#dummy.rs#src/bin/rust-prometheus-http-exporter.rs#' Cargo.toml
# RUN find
#COPY Cargo.toml ./
COPY src ./src

RUN cat Cargo.toml

RUN echo "[[bin]]" >> Cargo.toml
RUN echo "name = \"ssh-prometheus-exporter\"" >> Cargo.toml
RUN echo "path = \"src/bin/rust-prometheus-http-exporter.rs\"" >> Cargo.toml

RUN cat Cargo.toml
#RUN find
#RUN ls -la
#COPY . .
#RUN cargo build --release
RUN cargo build --release

# Copy the source and build the application.
#COPY Cargo.toml Cargo.lock ./
#COPY src ./src
#RUN cargo install --target aarch64-unknown-linux-musl --path .
#RUN cargo install --path .
RUN ls -la ${WORKDIR}/target/release

# Copy the statically-linked binary into a scratch container.
#FROM scratch
#FROM alpine
FROM rust:1.44-buster
ARG PROJECT_NAME="ssh-prometheus-exporter"
ARG BASEDIR=/var/development/src
ARG WORKDIR=${BASEDIR}/${PROJECT_NAME}
COPY --from=build ${WORKDIR}/target/release/${PROJECT_NAME} rust-binary
USER 1000
CMD ["./rust-binary"]
