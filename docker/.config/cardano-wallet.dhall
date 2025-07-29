let Prelude = (./prelude.dhall).Prelude

let CardanoWalletService =
      { Type =
          { image : Text
          , restart : Text
          , ports : Optional (List Text)
          , volumes : List Text
          , entrypoint : List Text
          , command : List Text
          , depends_on : Prelude.Map.Type Text { condition : Text }
          , healthcheck :
              { test : List Text
              , interval : Text
              , timeout : Text
              , retries : Natural
              }
          }
      , default =
        { image = "cardanofoundation/cardano-wallet:2025.3.31"
        , restart = "always"
        , entrypoint = [] : List Text
        , command =
          [ "bash"
          , "-c"
          , ''
            cardano-wallet serve \
              --database /wallet/db \
              --node-socket /node/testnet/socket/node1/sock \
              --testnet /node/testnet/byron-genesis.json \
              --listen-address 0.0.0.0
            ''
          ]
        , healthcheck =
          { test = [ "CMD-SHELL", "cardano-wallet network information" ]
          , interval = "2s"
          , timeout = "5s"
          , retries = 30
          }
        }
      }

let Options =
      { Type =
          { hostPort : Optional Natural
          , testnetVolume : Text
          , cardanoNodeHost : Text
          }
      , default.hostPort = None Natural
      }

let makeWalletService =
      \(options : Options.Type) ->
        CardanoWalletService::{
        , volumes = [ "${options.testnetVolume}:/node/testnet" ]
        , ports =
            Prelude.Optional.map
              Natural
              (List Text)
              (\(n : Natural) -> [ Prelude.Natural.show n ])
              options.hostPort
        , depends_on =
          [ { mapKey = options.cardanoNodeHost
            , mapValue.condition = "service_healthy"
            }
          ]
        }

in  { Options, makeWalletService }
