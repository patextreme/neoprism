val scala3Version = "3.3.6"

lazy val root = project
  .in(file("."))
  .settings(
    name := "prism-test",
    version := "0.1.0-SNAPSHOT",
    scalaVersion := scala3Version,
    scalacOptions := Seq(
      "-Xsource:3",
      "-feature",
      "-deprecation",
      "-unchecked",
      "-Wunused:all"
    ),
    Compile / PB.targets := Seq(
      scalapb.gen() -> (Compile / sourceManaged).value / "scalapb"
    ),
    Compile / PB.protoSources := Seq(
      baseDirectory.value / ".." / ".." / "lib" / "did-prism" / "proto",
      (Compile / resourceDirectory).value // includes scalapb codegen package wide config
    ),
    libraryDependencies ++= Seq(
      "com.thesamet.scalapb" %% "scalapb-runtime" % scalapb.compiler.Version.scalapbVersion % "protobuf",
      "com.thesamet.scalapb" %% "scalapb-runtime-grpc" % scalapb.compiler.Version.scalapbVersion
    ),
    libraryDependencies ++= Seq(
      "dev.zio" %% "zio" % "2.1.20"
    )
  )
