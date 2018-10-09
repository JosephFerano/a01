all: build

dist: build
	@rm assig-01-scheduler.tar.gz
	@tar -czf assig-01-scheduler.tar.gz src/ README.md target/release/client target/release/server

build:
	cargo build --release
