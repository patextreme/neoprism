package services

_#NeoprismVersion: "0.1.0"

_#NeoprismBaseService: {
	image:   "hyperledgeridentus/identus-neoprism:\(_#NeoprismVersion)"
	restart: "always"
	ports: ["8080:8080"]
	depends_on: ["db"]
	environment: {
		RUST_LOG:               "oura=warn,tracing::span=warn,info"
		NPRISM_DB_URL:          "postgres://postgres:postgres@db:5432/postgres"
		NPRISM_CARDANO_NETWORK: "mainnet"
		...
	}
}

#NeoprismServiceBuilder: {
	in: {source: "oura" | "dbsync"}

	_extraEnvs: {
		if in.source == "oura" {
			NPRISM_CARDANO_ADDR: "backbone.mainnet.cardanofoundation.org:3001"
		}
		if in.source == "dbsync" {
			NPRISM_DBSYNC_URL: "<DBSYNC_URL>"
		}
	}

	out: _#NeoprismBaseService & {
		environment: _extraEnvs
	}
}
