let indexer = ./indexer.dhall

let db = ./db.dhall

in  { basic.services
      =
      { indexer-node = indexer.makeIndexerNodeService indexer.ouraOption
      , db = db.makeDbService db.Options::{=}
      }
    , dbsync.services
      =
      { indexer-node = indexer.makeIndexerNodeService indexer.dbSyncOption
      , db = db.makeDbService db.Options::{=}
      }
    }
