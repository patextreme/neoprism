let Prelude = (./prelude.dhall).Prelude

let version = (./prelude.dhall).neoPrismVersion

let IndexerNodeService =
      { Type =
          { image : Text
          , restart : Text
          , ports : List Text
          , command : List Text
          , depends_on : Prelude.Map.Type Text { condition : Text }
          , environment : Prelude.Map.Type Text Text
          }
      , default =
        { image = "identus-neoprism:${version}"
        , restart = "always"
        , depends_on = [] : Prelude.Map.Type Text { condition : Text }
        , environment = [] : Prelude.Map.Type Text Text
        }
      }

let DltSource = < Relay : Text | DbSync : Text >

let DltSink =
      { Type =
          { walletBaseUrl : Text
          , walletId : Text
          , walletPassphrase : Text
          , walletPaymentAddress : Text
          }
      , default = {=}
      }

let Options =
      { Type =
          { hostPort : Natural
          , dbHost : Text
          , network : Text
          , dltSource : DltSource
          , dltSink : Optional DltSink.Type
          , confirmationBlocks : Optional Natural
          }
      , default =
        { hostPort = 8080
        , dbHost = "db"
        , network = "mainnet"
        , dltSink = None DltSink.Type
        , confirmationBlocks = None Natural
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

        let extraEnvs =
                merge
                  { None = [] : Prelude.Map.Type Text Text
                  , Some =
                      \(n : Natural) ->
                        toMap
                          { NPRISM_CONFIRMATION_BLOCKS = Prelude.Natural.show n
                          }
                  }
                  options.confirmationBlocks
              # merge
                  { Relay =
                      \(addr : Text) ->
                        toMap { NPRISM_CARDANO_RELAY_ADDR = addr }
                  , DbSync =
                      \(url : Text) -> toMap { NPRISM_CARDANO_DBSYNC_URL = url }
                  }
                  options.dltSource
              # merge
                  { None = [] : Prelude.Map.Type Text Text
                  , Some =
                      \(sink : DltSink.Type) ->
                        toMap
                          { NPRISM_CARDANO_WALLET_BASE_URL = sink.walletBaseUrl
                          , NPRISM_CARDANO_WALLET_WALLET_ID = sink.walletId
                          , NPRISM_CARDANO_WALLET_PASSPHRASE =
                              sink.walletPassphrase
                          , NPRISM_CARDANO_WALLET_PAYMENT_ADDR =
                              sink.walletPaymentAddress
                          }
                  }
                  options.dltSink

        let command =
              if    Prelude.Optional.null DltSink.Type options.dltSink
              then  "indexer"
              else  "standalone"

        in  IndexerNodeService::{
            , ports = [ "${Prelude.Natural.show options.hostPort}:8080" ]
            , environment = mandatoryIndexerNodeEnvs # extraEnvs
            , command = [ command ]
            , depends_on =
              [ { mapKey = options.dbHost
                , mapValue.condition = "service_healthy"
                }
              ]
            }

in  { Options, makeIndexerNodeService, DltSource, DltSink }
