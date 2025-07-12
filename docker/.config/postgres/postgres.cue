package postgres

_#PostgresVersion: "16"

_#BaseService: {
	image:   "postgres:\(_#PostgresVersion)"
	restart: "always"
	ports: ["5432:5432"]
	environment: {
		POSTGRES_USER:     "postgres"
		POSTGRES_PASSWORD: "postgres"
		POSTGRES_DB:       "postgres"
	}
}

#ServiceBuilder: {
	in: {}
	out: _#BaseService
}
