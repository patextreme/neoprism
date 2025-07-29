let Prelude = (./prelude.dhall).Prelude

let PrismNodeService =
      { Type =
          { image : Text
          , restart : Text
          , ports : Optional (List Text)
          , depends_on : Prelude.Map.Type Text { condition : Text }
          , environment : Prelude.Map.Type Text Text
          }
      , default =
        { image = "ghcr.io/input-output-hk/prism-node:2.6.0"
        , restart = "always"
        , ports = None (List Text)
        , depends_on = [] : Prelude.Map.Type Text { condition : Text }
        , environment = [] : Prelude.Map.Type Text Text
        }
      }

let Options =
      { Type =
          { nodeDbHost : Text
          , dbSyncDbHost : Text
          , bootstrapTestnetHost : Text
          , walletApiHost : Text
          , walletApiPort : Natural
          , walletPassphrase : Text
          , walletId : Text
          , walletPaymentAddress : Text
          , hostPort : Optional Natural
          }
      , default = { walletApiPort = 8090, hostPort = None Natural }
      }

let makePrismNodeService =
      \(options : Options.Type) ->
        PrismNodeService::{
        , ports =
            Prelude.Optional.map
              Natural
              (List Text)
              (\(p : Natural) -> [ "${Prelude.Natural.show p}:50053" ])
              options.hostPort
        , environment = toMap
            { NODE_PSQL_HOST = "${options.nodeDbHost}:5432"
            , NODE_PSQL_DATABASE = "postgres"
            , NODE_PSQL_USERNAME = "postgres"
            , NODE_PSQL_PASSWORD = "postgres"
            , NODE_LEDGER = "cardano"
            , NODE_CARDANO_CONFIRMATION_BLOCKS = "1"
            , NODE_REFRESH_AND_SUBMIT_PERIOD = "1s"
            , NODE_MOVE_SCHEDULED_TO_PENDING_PERIOD = "1s"
            , NODE_CARDANO_NETWORK = "testnet"
            , NODE_CARDANO_WALLET_PASSPHRASE = options.walletPassphrase
            , NODE_CARDANO_WALLET_ID = options.walletId
            , NODE_CARDANO_PAYMENT_ADDRESS = options.walletPaymentAddress
            , NODE_CARDANO_WALLET_API_HOST = options.walletApiHost
            , NODE_CARDANO_WALLET_API_PORT =
                Prelude.Natural.show options.walletApiPort
            , NODE_CARDANO_PRISM_GENESIS_BLOCK = "0"
            , NODE_CARDANO_DB_SYNC_HOST = "${options.dbSyncDbHost}:5432"
            , NODE_CARDANO_DB_SYNC_DATABASE = "postgres"
            , NODE_CARDANO_DB_SYNC_USERNAME = "postgres"
            , NODE_CARDANO_DB_SYNC_PASSWORD = "postgres"
            }
        , depends_on =
          [ { mapKey = options.nodeDbHost
            , mapValue.condition = "service_healthy"
            }
          , { mapKey = options.dbSyncDbHost
            , mapValue.condition = "service_healthy"
            }
          , { mapKey = options.bootstrapTestnetHost
            , mapValue.condition = "service_completed_successfully"
            }
          ]
        }

in  { Options, makePrismNodeService }
