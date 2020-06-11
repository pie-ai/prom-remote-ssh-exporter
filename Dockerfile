FROM rust:1.31

WORKDIR /usr/src/myapp
COPY . .

RUN cargo install --path .
RUN find

CMD ["myapp"]
