FROM rust:1.42

WORKDIR /usr/src/myapp
COPY . .

RUN cargo install --path .
RUN find

CMD ["myapp"]
