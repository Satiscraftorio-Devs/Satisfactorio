SHELL := /bin/bash

build: fmt
	cargo build >/dev/null 2>&1

server: build
	cargo run -q --bin server | tee logs/server.txt &

client: build
	cargo run -q --bin client | tee logs/client.txt

clean-logs:
	rm logs/* -rf

fmt:
	cargo fmt

check: fmt
	cargo check

run: clean-logs server client killall

killall:
	pkill -f target/debug/server 2>/dev/null
	pkill -f target/release/server 2>/dev/null
	pkill -f target/debug/client 2>/dev/null
	pkill -f target/release/client 2>/dev/null

kill: killall


clean-code:
	cargo fix --bin "server" -p server
	cargo fix --bin "client" -p client
