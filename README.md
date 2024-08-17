# Overview

Prism Node implementation in rust according to the [Prism DID Method](https://github.com/input-output-hk/prism-did-method-spec/blob/main/w3c-spec/PRISM-method.md).

# Quickstart

## Public demo instance

A public instance of neoprism is hosted at [https://neoprism.patlo.dev](https://neoprism.patlo.dev)

## Self hosting

Start the node and sync block metadata from relay node `relays-new.cardano-mainnet.iohk.io:3001`

```
cd docker
docker-compose up --build
```

WebUI is available at `http://localhost:8080`
