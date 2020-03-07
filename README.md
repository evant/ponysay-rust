# ponysay-rust

A barebones port of [ponysay](https://github.com/erkin/ponysay) to rust.

## Install

```
git clone https://github.com/evant/ponysay-rust.git
cd ponysay-rust
make
sudo make install
```

This will install to `/usr/bin/ponysay` by default. And place the pony files in `/usr/share/ponysay/`. 
If you want to change the prefix, you can do:

```
make PREFIX=/usr/local
sudo make install PREFIX=/usr/local
```

## Supported features

```
ponysay -l
ponysay -q
echo 'Hello, Equestria!' | ponysay
```
