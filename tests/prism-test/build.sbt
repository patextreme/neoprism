val scala3Version = "3.3.6"

val D = new {
  val scalaPbDeps: Seq[ModuleID] = Seq(
    "com.thesamet.scalapb" %% "scalapb-runtime" % scalapb.compiler.Version.scalapbVersion % "protobuf",
    "com.thesamet.scalapb" %% "scalapb-runtime-grpc" % scalapb.compiler.Version.scalapbVersion
  )

  val apolloDeps: Seq[ModuleID] = Seq(
    "org.hyperledger.identus" % "apollo-jvm" % "1.8.0" exclude (
      "net.jcip",
      "jcip-annotations"
    ), // Exclude because of license
    "com.github.stephenc.jcip" % "jcip-annotations" % "1.0-1" % Runtime // Replace for net.jcip % jcip-annotations"
  )

  val deps: Seq[ModuleID] = Seq(
    "dev.zio" %% "zio" % "2.1.20",
    "dev.optics" %% "monocle-core" % "3.1.0",
    "dev.optics" %% "monocle-macro" % "3.1.0",
    "io.grpc" % "grpc-netty" % "1.73.0"
  )
}

lazy val root = project
  .in(file("."))
  .settings(
    name := "prism-test",
    version := "0.1.0-SNAPSHOT",
    scalaVersion := scala3Version,
    scalacOptions := Seq(
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
    libraryDependencies ++= D.scalaPbDeps ++ D.apolloDeps ++ D.deps
  )
