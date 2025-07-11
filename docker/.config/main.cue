package main

import Svc "org.hyperledger.identus.neoprism/services"

basic: {
	services: {
		"indexer-node": (Svc.#NeoprismServiceBuilder & {in: {source: "oura"}}).out
		"db": Svc.#PostgresServiceBuilder.out
	}
}

dbsync: {
	services: {
		"indexer-node": (Svc.#NeoprismServiceBuilder & {in: {source: "dbsync"}}).out
		"db": Svc.#PostgresServiceBuilder.out
	}
}
