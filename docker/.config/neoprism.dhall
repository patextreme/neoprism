let Prelude = (./prelude.dhall).Prelude

let version = (./prelude.dhall).neoPrismVersion

let IndexerNodeService =
      { Type =
          { image : Text
          , restart : Text
          , ports : List Text
          , depends_on : Prelude.Map.Type Text { condition : Text }
          , environment : Prelude.Map.Type Text Text
          }
      , default =
        { image = "hyperledgeridentus/identus-neoprism:${version}"
        , restart = "always"
        , ports = [ "8080:8080" ]
        , depends_on = [] : Prelude.Map.Type Text { condition : Text }
        , environment = [] : Prelude.Map.Type Text Text
        }
      }

let Options =
      { Type =
          { extraEnvs : Prelude.Map.Type Text Text
          , dbHost : Text
          , dbName : Text
          , dbPort : Natural
          , dbUser : Text
          , dbPassword : Text
          }
      , default =
        { dbHost = "db"
        , dbPort = 5432
        , dbName = "postgres"
        , dbUser = "postgres"
        , dbPassword = "postgres"
        , extraEnvs = [] : Prelude.Map.Type Text Text
        }
      }

let makeIndexerNodeService =
      \(options : Options.Type) ->
        let mandatoryIndexerNodeEnvs =
              toMap
                { RUST_LOG = "oura=warn,tracing::span=warn,info"
                , NPRISM_DB_URL =
                    "postgres://postgres:postgres@db:5432/postgres"
                , NPRISM_CARDANO_NETWORK = "mainnet"
                }

        in  IndexerNodeService::{
            , environment = mandatoryIndexerNodeEnvs # options.extraEnvs
            , depends_on =
              [ { mapKey = options.dbHost
                , mapValue.condition = "service_healthy"
                }
              ]
            }

in  { Options
    , makeIndexerNodeService
    , ouraOption = Options::{
      , extraEnvs = toMap
          { NPRISM_CARDANO_ADDR = "backbone.mainnet.cardanofoundation.org:3001"
          }
      }
    , dbSyncOption = Options::{
      , extraEnvs = toMap { NPRISM_DBSYNC_URL = "<TODO>" }
      }
    }
