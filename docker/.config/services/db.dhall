let Prelude = (../prelude.dhall).Prelude

let docker = ../docker.dhall

let image = "postgres:16"

let Options =
      { Type = { hostPort : Optional Natural }
      , default.hostPort = None Natural
      }

let mkService =
      \(options : Options.Type) ->
        docker.Service::{
        , image
        , ports =
            Prelude.Optional.map
              Natural
              (List Text)
              (\(p : Natural) -> [ "${Prelude.Natural.show p}:5432" ])
              options.hostPort
        , environment = Some
            ( toMap
                { POSTGRES_DB = "postgres"
                , POSTGRES_PASSWORD = "postgres"
                , POSTGRES_USER = "postgres"
                }
            )
        , healthcheck = Some docker.Healthcheck::{
          , test = [ "CMD", "pg_isready", "-U", "postgres" ]
          }
        }

in  { mkService, Options }
