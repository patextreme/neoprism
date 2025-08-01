let Prelude = (./prelude.dhall).Prelude

let Healthcheck =
      { Type =
          { test : List Text
          , interval : Text
          , timeout : Text
          , retries : Natural
          }
      , default = { interval = "2s", timeout = "5s", retries = 30 }
      }

let ServiceCondition =
      { Type = { mapKey : Text, mapValue : { condition : Text } }
      , default.mapValue.condition = "service_healthy"
      }

let mkServiceCondition =
      \(cond : Text) ->
      \(name : Text) ->
        ServiceCondition::{ mapKey = name, mapValue.condition = cond }

let Service =
      { Type =
          { image : Text
          , restart : Optional Text
          , ports : Optional (List Text)
          , command : Optional (List Text)
          , entrypoint : Optional (List Text)
          , environment : Optional (Prelude.Map.Type Text Text)
          , volumes : Optional (List Text)
          , depends_on : Optional (List ServiceCondition.Type)
          , healthcheck : Optional Healthcheck.Type
          }
      , default =
        { restart = Some "always"
        , ports = None (List Text)
        , command = None (List Text)
        , entrypoint = None (List Text)
        , environment = None (Prelude.Map.Type Text Text)
        , volumes = None (List Text)
        , depends_on = None (List ServiceCondition.Type)
        , healthcheck = None Healthcheck.Type
        }
      }

in  { Service, ServiceCondition, mkServiceCondition, Healthcheck }
