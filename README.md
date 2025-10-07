# Kaze Backend

## Description

This project is the implementation of the Backend of Renta (previously Kaze)  project - a realtor-free reantal marketplace
that moves all property management to your phone. It makes use of public registries to automatically handle all the documents needed. 

## Prerequisites

1. Linux x86-64 platform (for EUSignCP compability)
2. Rust installed.
3. OpenSSL installed
4. Private keys and certificate inserted in `.libs/eusign/pkey` and `./libs/eusign/certificates`, respectively.

## How To Run

### Cargo

Clone the repository. Then run:

```shell
cargo build --release
cargo run --release -- server
```

### Docker

```shell
docker compose up -d
```

### Systemd

First, move `deploy/systemd/kaze.service` to your systemd containers directory (usually `etc/systemd/system/` on Linux).

Then:

```shell
systemctl daemon-reload
systemctl start kaze.service
```
