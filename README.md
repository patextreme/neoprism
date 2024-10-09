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

## Self hosting

Start the node and sync block metadata from relay node `backbone.mainnet.cardanofoundation.org:3001`

```bash
cd docker
docker-compose up --build
```

WebUI is available at `http://localhost:8080`

## Run from Cargo

Cargo alias `node` can be used to quickly see all CLI options.

```bash
cargo node -h
```

## About this project

This project began as a toy project with limited capabilities.
It serves as a proof of concept to enhance efficiency and simplify the Hyperledger Identus stack by eliminating the DB-sync requirement in IOG's PRISM Node implementation.
While still far from perfect, there is a roadmap and a proposal in [Project Catalyst Fund13](https://cardano.ideascale.com/c/idea/129248) to make it on par with the original PRISM Node,
with the potential to replace the original PRISM Node with NeoPRISM in the future.

## Architecture

TODO
