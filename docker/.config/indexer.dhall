let Prelude = (./prelude.dhall).Prelude

let IndexerNodeService =
      { Type =
          { image : Text
          , restart : Text
          , ports : List Text
          , depends_on : List Text
          , environment : Prelude.Map.Type Text Text
          }
      , default =
        { image = "hyperledgeridentus/identus-neoprism:0.1.0"
        , restart = "always"
        , ports = [ "8080:8080" ]
        , depends_on = [] : List Text
        , environment = [] : Prelude.Map.Type Text Text
        }
      }

let Options =
      { Type = { extraEnvs : Prelude.Map.Type Text Text, dbHost : Text }
      , default = { dbHost = "db", extraEnvs = [] : Prelude.Map.Type Text Text }
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
            , depends_on = [ options.dbHost ]
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
