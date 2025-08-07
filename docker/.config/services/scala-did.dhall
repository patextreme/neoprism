let Prelude = (../prelude.dhall).Prelude

let docker = ../docker.dhall

let image = "prism-node-fastsync:latest"

let Options =
      { Type = { hostPort : Optional Natural }
      , default.hostPort = None Natural
      }

let mkService =
      \(options : Options.Type) ->
        docker.Service::{
        , image
        , ports =
            Prelude.Optional.map
              Natural
              (List Text)
              (\(p : Natural) -> [ "${Prelude.Natural.show p}:8980" ])
              options.hostPort
        , entrypoint = Some [ "/bin/scala-did-node" ]
        }

in  { Options, mkService }
