package services

_#PostgresVersion: "16"

_#PostgresBaseService: {
	image:   "postgres:\(_#PostgresVersion)"
	restart: "always"
	ports: ["5432:5432"]
	environment: {
		POSTGRES_USER:     "postgres"
		POSTGRES_PASSWORD: "postgres"
		POSTGRES_DB:       "postgres"
	}
}

#PostgresServiceBuilder: {
	in: {}
	out: _#PostgresBaseService
}
