#!/usr/bin/make -f
# demon Makefile

HOST=$(HOSTNAME)

DOCS=docs
CFG=hack/cfg
SCRIPTS=scripts
USER_CONFIG=$(HOME)/.config/dm
BIN_DIR=/usr/bin
ETC=/etc/dm.json

_: build

help:
	@echo TODO: helpfile
build:
	cargo build --all

fmt:
	cargo fmt --all

	hg add .	;\
	hg commit -m "$(shell read -p 'Commit message: ' msg; echo $$msg)"

mirror:
	git add . ;\
	git commit -m "$(shell read -p 'Commit message: ' msg; echo $$msg)"  ;\
	git push -f --set-upstream origin master

serve: $(DOCS)
	darkhttpd $(DOCS) --addr 0.0.0.0 --index index.html --no-keepalive --no-server-id

clean:
	rm -rf target* Cargo.lock* bin/lambda/stripe/package-lock.json* bin/lambda/stripe/node_modules*
