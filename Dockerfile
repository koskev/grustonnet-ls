FROM debian
COPY ./target/release/grustonnet-ls /usr/local/bin/grustonnet-ls
COPY ./target/release/grustonnet-lint /usr/local/bin/grustonnet-lint
CMD ["grustonnet-ls"]

