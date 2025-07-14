let Prelude = (./prelude.dhall).Prelude

let DbService =
      { Type =
          { image : Text
          , restart : Text
          , ports : List Text
          , environment : Prelude.Map.Type Text Text
          }
      , default =
        { image = "postgres:16"
        , restart = "always"
        , ports = [ "5432:5432" ]
        , environment = toMap
            { POSTGRES_DB = "postgres"
            , POSTGRES_PASSWORD = "postgres"
            , POSTGRES_USER = "postgres"
            }
        }
      }

let Options = { Type = {}, default = {=} }

let makeDbService = \(options : Options.Type) -> DbService::{=}

in  { makeDbService, Options }
