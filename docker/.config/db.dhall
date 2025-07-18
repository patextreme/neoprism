let Prelude = (./prelude.dhall).Prelude

let DbService =
      { Type =
          { image : Text
          , restart : Text
          , ports : Optional (List Text)
          , environment : Prelude.Map.Type Text Text
          , healthcheck :
              { test : List Text
              , interval : Text
              , timeout : Text
              , retries : Natural
              }
          }
      , default =
        { image = "postgres:16"
        , restart = "always"
        , ports = None (List Text)
        , healthcheck =
          { test = [ "CMD-SHELL", "pg_isready -U postgres" ]
          , interval = "5s"
          , timeout = "5s"
          , retries = 10
          }
        , environment = toMap
            { POSTGRES_DB = "postgres"
            , POSTGRES_PASSWORD = "postgres"
            , POSTGRES_USER = "postgres"
            }
        }
      }

let Options =
      { Type = { hostPort : Optional Natural }
      , default.hostPort = None Natural
      }

let makeDbService =
      \(options : Options.Type) ->
        DbService::{
        , ports =
            Prelude.Optional.map
              Natural
              (List Text)
              (\(p : Natural) -> [ "${Prelude.Natural.show p}:5432" ])
              options.hostPort
        }

in  { makeDbService, Options }
