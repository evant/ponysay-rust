# ponysay-rust

A barebones port of [ponysay](https://github.com/erkin/ponysay) to rust.

## Why?

ponysay is written in python, I found it frustratingly slow to run. As I only use a fraction of it's features, I re-wrote
the parts I use in rust.

## Install

Requires rust to build, you can obtain it with [rustup](https://rustup.rs/).

```
git clone https://github.com/evant/ponysay-rust.git
cd ponysay-rust
make
sudo make install
```

This will install to `/usr/local/bin/ponysay` by default. And place the pony files in `/usr/local/share/ponysay/`. 
If you want to change the prefix, you can do:

```
make PREFIX=/my/prefix/dir
sudo make install PREFIX=/my/prefix/dir
```

## Supported features

```
ponysay -l
ponysay -q
echo 'Hello, Equestria!' | ponysay
ponysay -f pinkie -- 'Lets have a party!'
```
