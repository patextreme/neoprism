let Prelude = (../prelude.dhall).Prelude

let docker = ../docker.dhall

let image = "ghcr.io/intersectmbo/cardano-db-sync:13.6.0.5"

let Options =
      { Type =
          { testnetVolume : Text
          , cardanoNodeHost : Text
          , configFile : Text
          , dbHost : Text
          }
      , default = {=}
      }

let mkService =
      \(options : Options.Type) ->
        docker.Service::{
        , image
        , environment = Some
            ( toMap
                { POSTGRES_HOST = options.dbHost
                , POSTGRES_DB = "postgres"
                , POSTGRES_PORT = "5432"
                , POSTGRES_USER = "postgres"
                , POSTGRES_PASSWORD = "postgres"
                }
            )
        , command = Some
          [ "--config"
          , "/config/dbsync-config.yaml"
          , "--socket-path"
          , "/node/testnet/socket/node1/sock"
          , "--force-indexes"
          ]
        , volumes = Some
          [ "${options.testnetVolume}:/node/testnet"
          , "${options.configFile}:/config/dbsync-config.yaml"
          ]
        , depends_on = Some
          [ docker.ServiceCondition.healthy options.cardanoNodeHost
          , docker.ServiceCondition.healthy options.dbHost
          ]
        }

in  { Options, mkService }
