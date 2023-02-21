#! /bin/sh/ 

docker run -t --rm rust:latest bash -c "cargo new program && cd program && cargo run"