# Overview

This is a PRISM Node implementation in Rust according to the [PRISM DID Method](https://github.com/input-output-hk/prism-did-method-spec/blob/main/w3c-spec/PRISM-method.md).

## PRISM DID Introduction

If you are already familiar with DID and PRISM DID method, skip this part.

A [Decentralized Identifier (DID)](https://www.w3.org/TR/did-core/) is a type of URI used as a stable identifier for a resource known as a DID Document.
[A DID Document](https://www.w3.org/TR/did-core/#dfn-did-documents) contains a collection of public keys and optionally some other resources that applications can use.
Various types of DID implementations, called DID Methods, exist.
PRISM DID is one such method, where operations that modify the DID Document are published to the Cardano blockchain.

Published PRISM DID operations are encoded in [protobuf format](https://github.com/input-output-hk/prism-did-method-spec/blob/main/w3c-spec/PRISM-method.md#appendix-b-protobuf-models) and must adhere to the rules outlined in the PRISM DID specification.
The protobuf-encoded operations are embedded in Cardano transaction metadata and publicly available for all parties to validate.

The PRISM Node's role is to follow the Cardano blockchain, read, parse, and validate published PRISM DID operations.
It functions as an indexer, allowing applications to query DIDs and retrieve the corresponding DID Documents.
This process, known as [DID resolution](https://www.w3.org/TR/did-core/#resolution), returns the DID Document in the [W3C-compliant format](https://www.w3.org/TR/did-core/#representations), so applications don't need to know about the details of each DID Method.

It is recommended to check out this [protocol high level description.](https://github.com/input-output-hk/prism-did-method-spec/blob/main/w3c-spec/PRISM-method.md#high-level-protocol-description)

# Quickstart

## Public demo instance

A public instance of neoprism is hosted at [https://neoprism.patlo.dev](https://neoprism.patlo.dev).  
A public preprod instance is also available at [https://neoprism-preprod.patlo.dev](https://neoprism-preprod.patlo.dev).

## Self-hosting

Start the node and sync block metadata from relay node `backbone.mainnet.cardanofoundation.org:3001`

```bash
cd docker
docker-compose up --build
```

WebUI is available at `http://localhost:8080`

Resolver endpoint is availabe at `http://localhost:8080/api/dids/<did>`


# Development guide

This project uses Nix for the local development environment and artifact packaging.
Follow the instructions [here](https://nixos.org/download/#download-nix) to install Nix—it's all you need!

__Entering the development shell__

If you already have `cargo` and other required dependencies (e.g. `protoc`) installed, you can use your own environment.
Feel free to check [nix shell](./nix/devShells.nix) to see the required dependencies and port it to your own environment.

A recommended approach is to use `nix develop` command to enter the development shell.
This way, the development shell is consistent and the same version of the libraries are used to build and test.


```bash
nix develop

# you can now run command like "cargo version"
```
Note that you may need to enable experiment flake command. Please follow the intructions [here](https://nixos.wiki/wiki/Flakes).

Additionally, you can use `--unset <ENV>` to disable host environment variable to make development shell more pure.
For example:

```bash
nix develop --unset PATH
```

to disable all binaries available on host `PATH`.


## Development quickstart

Spinning up services in dev shell

```bash
nix develop --unset PATH
npm install
dbUp
runNode
```

Cleaning up services in dev shell

```bash
dbDown
```

## Frequently used commands

These are commands you can run outside the development shell

| command                                                     | description                                                        |
|-------------------------------------------------------------|--------------------------------------------------------------------|
| `nix flake check`                                           | Run checks to see if it will pass the CI (CI runs this check)      |
| `nix build .#indexer-docker`                                | Use nix to build the docker image (output available at `./result`) |
| `nix build .#indexer-docker && cat ./result \| docker load` | Use nix to build the docker image and load it using docker         |

Assuming you are in the development shell, these are frequently used commands.

| command                         | description                                    |
|---------------------------------|------------------------------------------------|
| `npm install`                   | Install the npm dependencies (first time only) |
| `cargo build`                   | Build the cargo workspace                      |
| `cargo clean`                   | Clean the cargo workspace                      |
| `cargo r -p indexer-node -- -h` | See `indexer-node` service CLI options         |
| `cargo test --all-features`     | Run tests which enable all crate features      |

And these are some scripts provided by the shell to automate local dev workflow

| command                         | description                                                        |
|---------------------------------|--------------------------------------------------------------------|
| `format`                        | Run formatter on everything                                        |
| `build`                         | Building the whole project                                         |
| `buildAssets`                   | Building the WebUI assets (css, javascript, static assets)         |
| `dbUp`                          | Spin up the local database                                         |
| `dbDown`                        | Tear down the local database                                       |
| `pgDump`                        | Dump the local database to `postgres.dump` file                    |
| `pgRestore`                     | Restore the local database from `postgres.dump` file               |
| `runNode`                       | Run the `indexer-node` connecting to the local database            |
| `runNode --cardano-addr <ADDR>` | Run the `indexer-node` connecting to the cardano relay at `ADDR`   |
| `runNode --dbsync-url <URL>`    | Run the `indexer-node` connecting to the DB Sync instance at `URL` |
