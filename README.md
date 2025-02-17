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

## Run from Cargo

Cargo alias `node` can be used to quickly see all CLI options.

```bash
cargo node -h
```

## About this project

This project aims to enhance efficiency and simplify the [Hyperledger Identus](https://www.lfdecentralizedtrust.org/projects/identus) stack by eliminating the
DB-sync requirement in [IOG's PRISM Node implementation](https://github.com/input-output-hk/atala-prism).

The project is maintained on a casual basis as my personal time permits, but feel free to open an issue or a pull request.
Contributions are always welcome.

# Deployment

The workspace generates a single runnable binary under the `prism-node` crate with the following CLI options:

```
Usage: prism-node [OPTIONS] --db <DB_URL>

Options:
      --db <DB_URL>                Database URL (e.g. postgres://user:pass@host:5432/db)
      --skip-migration             Skip database migration on Node startup
      --cardano <CARDANO_ADDR>     Address of the Cardano node to consume events from. If not provided, the Node will not sync events from the Cardano node. (e.g. backbone.mainnet.cardanofoundation.org:3001)
      --network <CARDANO_NETWORK>  A Cardano network to connect to. This option must correlate with the network of the node address provided [default: mainnet]
      --address <ADDRESS>          Node HTTP server binding address [default: 0.0.0.0]
  -p, --port <PORT>                Node HTTP server listening port [default: 8080]
      --assets <ASSETS>            The directory containing the web-ui assets (CSS, Javascripts) [default: ./prism-node/assets]
  -h, --help                       Print help
```

The CLI options enable flexible deployment based on specific needs.

A fully functional deployment includes:

- `server` – Provides the web UI and a universal resolver endpoint.
- `sync worker` – Tracks the Cardano node, indexes DID operations, and stores them in the database.
- `database` – PostgreSQL for data storage.
- `cardano node` – Serves as the verifiable data registry for DID operations.

A system operator can use any trusted public `cardano node` instance.
However, for better performance, the `sync worker` should run close to the `cardano node`, ideally on the same network or host.

## Standalone mode

In this mode, all components run within a single container.
To enable this mode, use the following options:

- `--cardano=<address>` - to enable the sync worker

![](./docs/diagrams/deploy_standalone.png)

## High-availability mode

In this mode, the `sync worker` and `server` run in separate containers.
Since multiple containers are used, specific options must be set for each.

- One or more `server` can serve users and clients, connecting to a shared PostgreSQL (optionally in HA mode).
- A single `sync worker` indexes operations from Cardano.

for `worker` container, use

- `--cardano=<address>` - to enable the sync worker

for `server` container, use
- `--skip-migration` - to disable db migration

Make sure to not set `--cardano=<address>` for the `server` as
multiple sync workers are redundnat and may conflict the sync cursor.

![](./docs/diagrams/deploy_ha.png)

