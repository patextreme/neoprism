let Prelude = (./prelude.dhall).Prelude

let neoprism = ./neoprism.dhall

let db = ./db.dhall

let dbSync = ./cardano-dbsync.dhall

let cardanoNode = ./cardano-node.dhall

let cardanoWallet = ./cardano-wallet.dhall

let prismNode = ./prism-node.dhall

let cloudAgent = ./cloud-agent.dhall

in  { mainnet-dbsync.services
      =
      { db = db.makeDbService db.Options::{ hostPort = Some 5432 }
      , neoprism-indexer =
          neoprism.makeIndexerNodeService
            neoprism.Options::{
            , dltSource = neoprism.DltSource.DbSync "<DBSYNC_URL>"
            }
      }
    , mainnet-relay.services
      =
      { db = db.makeDbService db.Options::{ hostPort = Some 5432 }
      , neoprism-indexer =
          neoprism.makeIndexerNodeService
            neoprism.Options::{
            , dltSource =
                neoprism.DltSource.Relay
                  "backbone.mainnet.cardanofoundation.org:3001"
            }
      }
    , preprod-relay.services
      =
      { db = db.makeDbService db.Options::{ hostPort = Some 5432 }
      , neoprism-indexer =
          neoprism.makeIndexerNodeService
            neoprism.Options::{
            , network = "preprod"
            , dltSource =
                neoprism.DltSource.Relay
                  "preprod-node.play.dev.cardano.org:3001"
            }
      }
    , testnet-local =
        let networkMagic = 42

        let testnetVolume = "node-testnet"

        let cardanoNodeHost = "cardano-node"

        let walletId = "9263a1248b046fe9e1aabc4134b03dc5c3a7ee3d"

        let walletPassphrase = "super_secret"

        let walletPaymentAddress =
              "addr_test1qp83v2wq3z9mkcjj5ejlupgwt6tcly5mtmz36rpm8w4atvqd5jzpz23y8l4dwfd9l46fl2p86nmkkx5keewdevqxhlyslv99j3"

        in  { services =
              { cardano-node =
                  cardanoNode.makeNodeService
                    cardanoNode.Options::{ networkMagic, testnetVolume }
              , cardano-wallet =
                  cardanoWallet.makeWalletService
                    cardanoWallet.Options::{
                    , testnetVolume
                    , cardanoNodeHost
                    , hostPort = Some 8090
                    }
              , bootstrap-testnet =
                  cardanoNode.makeBootstrapService
                    cardanoNode.BootstrapOptions::{
                    , networkMagic
                    , testnetVolume
                    , cardanoNodeHost
                    , walletBaseUrl = "http://cardano-wallet:8090/v2"
                    , walletPassphrase
                    , walletPaymentAddress
                    , initWalletHurlFile = "./init-wallet.hurl"
                    }
              , cardano-dbsync =
                  dbSync.makeDbSyncService
                    dbSync.Options::{
                    , testnetVolume
                    , cardanoNodeHost
                    , dbHost = "db-dbsync"
                    , configFile = "./dbsync-config.yaml"
                    }
              , neoprism-standalone =
                  neoprism.makeIndexerNodeService
                    neoprism.Options::{
                    , dbHost = "db-neoprism"
                    , confirmationBlocks = Some 1
                    , dltSource =
                        neoprism.DltSource.DbSync
                          "postgresql://postgres:postgres@db-dbsync:5432/postgres"
                    , dltSink = Some neoprism.DltSink::{
                      , walletBaseUrl = "http://cardano-wallet:8090/v2"
                      , walletId
                      , walletPassphrase
                      , walletPaymentAddress
                      }
                    }
              , identus-prism-node =
                  prismNode.makePrismNodeService
                    prismNode.Options::{
                    , nodeDbHost = "db-prism-node"
                    , dbSyncDbHost = "db-dbsync"
                    , bootstrapTestnetHost = "bootstrap-testnet"
                    , walletApiHost = "cardano-wallet"
                    , walletPassphrase
                    , walletId
                    , walletPaymentAddress
                    }
              , identus-cloud-agent =
                  cloudAgent.makeCloudAgentService
                    cloudAgent.Options::{
                    , dbHost = "db-cloud-agent"
                    , prismNodeHost = "identus-prism-node"
                    }
              , db-prism-node = db.makeDbService db.Options::{=}
              , db-neoprism =
                  db.makeDbService db.Options::{ hostPort = Some 5432 }
              , db-dbsync =
                  db.makeDbService db.Options::{ hostPort = Some 5433 }
              , db-cloud-agent =
                      db.makeDbService db.Options::{=}
                  //  { environment = toMap
                          { POSTGRES_MULTIPLE_DATABASES = "pollux,connect,agent"
                          , POSTGRES_USER = "postgres"
                          , POSTGRES_PASSWORD = "postgres"
                          }
                      , volumes =
                        [ "./postgres/init_script.sh:/docker-entrypoint-initdb.d/init-script.sh"
                        , "./postgres/max_conns.sql:/docker-entrypoint-initdb.d/max_conns.sql"
                        ]
                      }
              }
            , volumes = toMap { node-testnet = {=} }
            }
    }
