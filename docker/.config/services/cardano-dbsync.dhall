let Prelude = (../prelude.dhall).Prelude

let DbSyncService =
      { Type =
          { image : Text
          , restart : Text
          , volumes : List Text
          , environment : Prelude.Map.Type Text Text
          , command : List Text
          , depends_on : Prelude.Map.Type Text { condition : Text }
          }
      , default =
        { image = "ghcr.io/intersectmbo/cardano-db-sync:13.6.0.5"
        , restart = "always"
        , command =
          [ "--config"
          , "/config/dbsync-config.yaml"
          , "--socket-path"
          , "/node/testnet/socket/node1/sock"
          , "--force-indexes"
          ]
        }
      }

let Options =
      { Type =
          { testnetVolume : Text
          , cardanoNodeHost : Text
          , configFile : Text
          , dbHost : Text
          }
      , default = {=}
      }

let makeDbSyncService =
      \(options : Options.Type) ->
        DbSyncService::{
        , environment = toMap
            { POSTGRES_HOST = options.dbHost
            , POSTGRES_DB = "postgres"
            , POSTGRES_PORT = "5432"
            , POSTGRES_USER = "postgres"
            , POSTGRES_PASSWORD = "postgres"
            }
        , volumes =
          [ "${options.testnetVolume}:/node/testnet"
          , "${options.configFile}:/config/dbsync-config.yaml"
          ]
        , depends_on =
          [ { mapKey = options.cardanoNodeHost
            , mapValue.condition = "service_healthy"
            }
          , { mapKey = options.dbHost, mapValue.condition = "service_healthy" }
          ]
        }

in  { Options, makeDbSyncService }
