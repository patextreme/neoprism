package org.hyperledger.identus.prismtest

import org.hyperledger.identus.prismtest.suite.CreateOperationSuite
import org.hyperledger.identus.prismtest.utils.TestUtils
import proto.prism_ssi.KeyUsage
import zio.*
import zio.http.Client
import zio.test.*
import zio.test.Assertion.*

object MainSpec extends ZIOSpecDefault, TestUtils:

  override def spec =
    val prismNodeSpec = suite("PRISM node suite")(CreateOperationSuite.values)
      .provide(NodeClient.grpc("localhost", 50053), NodeName.layer("prism-node"))

    val scalaDidSpec = suite("scala-did node suite")(CreateOperationSuite.values)
      .provide(NodeClient.grpc("localhost", 8980), NodeName.layer("scala-did"))

    val neoprismSpec = suite("NeoPRISM suite")(CreateOperationSuite.values)
      .provide(
        NodeClient.neoprism("localhost", 8080)("localhost", 8090),
        Client.default,
        NodeName.layer("neoprism")
      )

    (neoprismSpec + scalaDidSpec + prismNodeSpec).provide(Runtime.removeDefaultLoggers)
      @@ TestAspect.timed
      @@ TestAspect.withLiveEnvironment
      @@ TestAspect.parallelN(1)
