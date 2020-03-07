PREFIX = /usr

all: download-ponies target/release/ponysay

target/release/ponysay:
	PREFIX=$(PREFIX) cargo build --release

.PHONY: clean
clean:
	cargo clean
	rm -rf share

.PHONY: download-ponies
download-ponies:
	if [ -d "share/ponysay" ]; then \
  		cd share/ponysay && git pull; \
  	else \
  	  	mkdir share && cd share && git clone --depth 1 https://github.com/erkin/ponysay.git && cd ponysay && ln -s ponyquotes quotes; \
    fi

.PHONY: install
install: all
	mkdir -p $(DESTDIR)$(PREFIX)/share/ponysay/ponies
	mkdir -p $(DESTDIR)$(PREFIX)/share/ponysay/quotes
	mkdir -p $(DESTDIR)$(PREFIX)/bin
	cp -R share/ponysay/ponies/*.* $(DESTDIR)$(PREFIX)/share/ponysay/ponies/
	cp -R share/ponysay/quotes/*.* $(DESTDIR)$(PREFIX)/share/ponysay/quotes/
	install -s target/release/ponysay $(DESTDIR)$(PREFIX)/bin/ponysay
