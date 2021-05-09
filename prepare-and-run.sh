#! /usr/bin/env bash

set -e

ulimit -n 32768

for port in 20000 20001 20002 20003 20004
do
    PID=$(lsof -ti tcp:"$port" | xargs)
    if [ ! -z "$PID" ]
    then
        kill $PID
    fi
done

cargo build --release --manifest-path ./testserver/Cargo.toml

cargo build --release --manifest-path ./tokio_tcp_proxy/Cargo.toml

go build -o go_tcp_proxy/go_tcp_proxy go_tcp_proxy/main.go

cargo +nightly bench --manifest-path ./testserver/Cargo.toml