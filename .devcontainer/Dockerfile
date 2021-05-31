FROM rust:latest as builder

FROM builder as dev
RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
    && apt-get -y install --no-install-recommends git vim-tiny
RUN useradd -s /bin/bash -m vscode
RUN rustup component add rust-src

USER vscode
