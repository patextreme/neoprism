let Prelude = (./prelude.dhall).Prelude

let CloudAgentService =
      { Type =
          { image : Text
          , restart : Text
          , ports : List Text
          , depends_on : Prelude.Map.Type Text { condition : Text }
          , environment : Prelude.Map.Type Text Text
          }
      , default =
        { image = "hyperledgeridentus/identus-cloud-agent:2.0.0"
        , restart = "always"
        }
      }

let Options =
      { Type = { hostPort : Natural, dbHost : Text, prismNodeHost : Text }
      , default.hostPort = 8085
      }

let makeCloudAgentService =
      \(options : Options.Type) ->
        CloudAgentService::{
        , ports = [ "${Prelude.Natural.show options.hostPort}:8085" ]
        , environment = toMap
            { ADMIN_TOKEN = "admin"
            , AGENT_DB_HOST = options.dbHost
            , AGENT_DB_NAME = "agent"
            , AGENT_DB_PASSWORD = "postgres"
            , AGENT_DB_PORT = "5432"
            , AGENT_DB_USER = "postgres"
            , API_KEY_ENABLED = "false"
            , CONNECT_DB_HOST = options.dbHost
            , CONNECT_DB_NAME = "connect"
            , CONNECT_DB_PASSWORD = "postgres"
            , CONNECT_DB_PORT = "5432"
            , CONNECT_DB_USER = "postgres"
            , DIDCOMM_SERVICE_URL = "http://example.com/didcomm"
            , POLLUX_DB_HOST = options.dbHost
            , POLLUX_DB_NAME = "pollux"
            , POLLUX_DB_PASSWORD = "postgres"
            , POLLUX_DB_PORT = "5432"
            , POLLUX_DB_USER = "postgres"
            , POLLUX_STATUS_LIST_REGISTRY_PUBLIC_URL =
                "http://example.com/cloud-agent"
            , PRISM_NODE_HOST = options.prismNodeHost
            , PRISM_NODE_PORT = "50053"
            , REST_SERVICE_URL = "http://example.com/cloud-agent"
            , SECRET_STORAGE_BACKEND = "postgres"
            }
        , depends_on =
          [ { mapKey = options.dbHost, mapValue.condition = "service_healthy" }
          ]
        }

in  { Options, makeCloudAgentService }
