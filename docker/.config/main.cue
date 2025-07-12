package main

import (
	Pg "org.hyperledger.identus.neoprism/postgres"
	N "org.hyperledger.identus.neoprism/neoprism"
)

basic: {
	services: {
		"indexer-node": (N.#ServiceBuilder & {in: {source: "oura"}}).out
		"db": Pg.#ServiceBuilder.out
	}
}

dbsync: {
	services: {
		"indexer-node": (N.#ServiceBuilder & {in: {source: "dbsync"}}).out
		"db": Pg.#ServiceBuilder.out
	}
}
