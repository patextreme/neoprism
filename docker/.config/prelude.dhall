let Prelude =
      https://prelude.dhall-lang.org/v23.1.0/package.dhall
        sha256:931cbfae9d746c4611b07633ab1e547637ab4ba138b16bf65ef1b9ad66a60b7f

in  { Prelude
    , neoPrismVersion =
        let rawVersion = ../../version as Text

        let removeEof = Prelude.Text.replace "\n" ""

        let removeSpace = Prelude.Text.replace " " ""

        in  removeEof (removeSpace rawVersion)
    }
