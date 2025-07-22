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
        { image = "identus-neoprism:${version}" -- TODO: revert before PR
        , restart = "always"
        , depends_on = [] : Prelude.Map.Type Text { condition : Text }
        , environment = [] : Prelude.Map.Type Text Text
        }
      }

let Options =
      { Type =
          { extraEnvs : Prelude.Map.Type Text Text
          , hostPort : Natural
          , dbHost : Text
          , network : Text
          }
      , default =
        { hostPort = 8080
        , dbHost = "db"
        , network = "mainnet"
        , extraEnvs = [] : Prelude.Map.Type Text Text
        }
      }

let makeIndexerNodeService =
      \(options : Options.Type) ->
        let mandatoryIndexerNodeEnvs =
              toMap
                { RUST_LOG = "oura=warn,tracing::span=warn,info"
                , NPRISM_DB_URL =
                    "postgres://postgres:postgres@${options.dbHost}:5432/postgres"
                , NPRISM_CARDANO_NETWORK = options.network
                }

        in  IndexerNodeService::{
            , ports = [ "${Prelude.Natural.show options.hostPort}:8080" ]
            , environment = mandatoryIndexerNodeEnvs # options.extraEnvs
            , depends_on =
              [ { mapKey = options.dbHost
                , mapValue.condition = "service_healthy"
                }
              ]
            }

in  { Options, makeIndexerNodeService }
