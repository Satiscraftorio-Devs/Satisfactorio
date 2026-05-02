SHELL := /bin/bash

build: fmt
	RUSTFLAGS="-Awarnings" cargo build >/dev/null 2>&1

server-bg: build
	RUSTFLAGS="-Awarnings" cargo run -q --bin server -- --ip 127.0.0.1 --port 5000 | tee logs/server.txt &

server: build
	RUSTFLAGS="-Awarnings" cargo run -q --bin server -- --ip 127.0.0.1 --port 5000 |tee logs/server.txt


client: build
	RUSTFLAGS="-Awarnings" cargo run -q --bin client -- --ip 127.0.0.1 --port 5000 | tee logs/client.txt

clean-logs:
	rm logs/* -rf

fmt:
	cargo fmt

check: fmt
	cargo check

run: clean-logs server-bg client killall

killall:
	pkill -f target/debug/server 2>/dev/null
	pkill -f target/release/server 2>/dev/null
	pkill -f target/debug/client 2>/dev/null
	pkill -f target/release/client 2>/dev/null

kill: killall


clean-code:
	cargo fix --bin "server" -p server
	cargo fix --bin "client" -p client
