#!/bin/bash

PREFIX=/usr/local

TARGET=target/release/wl-bg-gallery-rs

mkdir -p ${PREFIX}/bin/
cp ${TARGET} ${PREFIX}/bin/

mkdir -p /etc/wl-bg-gallery/
cp config.toml /etc/wl-bg-gallery/

cp wl-bg-gallery.service /etc/systemd/user/
