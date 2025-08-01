let Prelude = (../prelude.dhall).Prelude

let docker = ../docker.dhall

let image = "cardanofoundation/cardano-wallet:2025.3.31"

let Options =
      { Type =
          { hostPort : Optional Natural
          , testnetVolume : Text
          , cardanoNodeHost : Text
          }
      , default.hostPort = None Natural
      }

let mkService =
      \(options : Options.Type) ->
        docker.Service::{
        , image
        , entrypoint = Some ([] : List Text)
        , command = Some
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
        , ports =
            Prelude.Optional.map
              Natural
              (List Text)
              (\(n : Natural) -> [ "${Prelude.Natural.show n}:8090" ])
              options.hostPort
        , volumes = Some [ "${options.testnetVolume}:/node/testnet" ]
        , healthcheck = Some docker.Healthcheck::{
          , test = [ "CMD-SHELL", "cardano-wallet network information" ]
          }
        , depends_on = Some
          [ docker.mkServiceCondition "service_healthy" options.cardanoNodeHost
          ]
        }

in  { Options, mkService }
