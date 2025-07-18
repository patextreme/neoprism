let Prelude = (./prelude.dhall).Prelude

let CardanoWalletService =
      { Type =
          { image : Text
          , restart : Text
          , volumes : List Text
          , entrypoint : List Text
          , command : List Text
          , depends_on : Prelude.Map.Type Text { condition : Text }
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
        }
      }

let Options =
      { Type = { testnetVolume : Text, cardanoNodeHost : Text }, default = {=} }

let makeWalletService =
      \(options : Options.Type) ->
        CardanoWalletService::{
        , volumes = [ "${options.testnetVolume}:/node/testnet" ]
        , depends_on =
          [ { mapKey = options.cardanoNodeHost
            , mapValue.condition = "service_healthy"
            }
          ]
        }

in  { Options, makeWalletService }
