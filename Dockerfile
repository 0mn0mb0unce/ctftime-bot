FROM scratch

COPY ./target/x86_64-unknown-linux-musl/release/onb-ctftime-bot /app

CMD ["/app"]
