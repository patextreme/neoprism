let Prelude = (./prelude.dhall).Prelude

let imageName = "cardano-testnet:latest"

let CardanoNodeService =
      { Type =
          { image : Text
          , command : List Text
          , environment : Prelude.Map.Type Text Text
          , volumes : List Text
          , healthcheck :
              { test : List Text
              , interval : Text
              , timeout : Text
              , retries : Natural
              }
          }
      , default =
        { image = imageName
        , command = [ "initTestnet" ]
        , healthcheck =
          { test = [ "CMD-SHELL", "cardano-cli query tip" ]
          , interval = "5s"
          , timeout = "5s"
          , retries = 10
          }
        }
      }

let Options =
      { Type = { networkMagic : Natural, testnetVolume : Text }, default = {=} }

let makeNodeService =
      \(options : Options.Type) ->
        CardanoNodeService::{
        , volumes = [ "${options.testnetVolume}:/node/testnet" ]
        , environment = toMap
            { CARDANO_NODE_SOCKET_PATH = "/node/testnet/socket/node1/sock"
            , CARDANO_NODE_NETWORK_ID =
                Prelude.Natural.show options.networkMagic
            }
        }

let BootstrapOptions =
      { Type =
          { networkMagic : Natural
          , testnetVolume : Text
          , cardanoNodeHost : Text
          , walletBaseUrl : Text
          , walletPassphrase : Text
          }
      , default = {=}
      }

let makeBootstrapService =
      \(options : BootstrapOptions.Type) ->
        { image = imageName
        , volumes = [ "${options.testnetVolume}:/node/testnet" ]
        , command =
          [ "bash"
          , "-c"
          , ''
            transactGenesis
            initWallet
            ''
          ]
        , environment = toMap
            { HURL_WALLET_BASE_URL = options.walletBaseUrl
            , HURL_WALLET_PASSPHRASE = options.walletPassphrase
            , CARDANO_NODE_SOCKET_PATH = "/node/testnet/socket/node1/sock"
            , CARDANO_NODE_NETWORK_ID =
                Prelude.Natural.show options.networkMagic
            }
        , depends_on =
          [ { mapKey = options.cardanoNodeHost
            , mapValue.condition = "service_healthy"
            }
          ]
        }

in  { Options, makeNodeService, BootstrapOptions, makeBootstrapService }
