SHELL := /bin/bash

build: fmt
	cargo build >/dev/null 2>&1

server: build
	cargo run -q --bin server &

client: build
	cargo run -q --bin client


fmt:
	cargo fmt

check: fmt
	cargo check

run: server client killall



killall: 
	pkill -f target/debug/server 2>/dev/null 
	pkill -f target/release/server 2>/dev/null
	pkill -f target/debug/client 2>/dev/null
	pkill -f target/release/client 2>/dev/null

kill: killall


clean: killall
	cargo clean
