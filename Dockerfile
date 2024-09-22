FROM scratch

COPY --chmod=755 ./onb-ctftime-bot /app

CMD ["/app"]
