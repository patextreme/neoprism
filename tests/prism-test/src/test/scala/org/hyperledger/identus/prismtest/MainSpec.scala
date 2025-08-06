package org.hyperledger.identus.prismtest

import org.hyperledger.identus.prismtest.suite.CreateOperationSuite
import org.hyperledger.identus.prismtest.suite.DeactivateOperationSuite
import org.hyperledger.identus.prismtest.suite.UpdateOperationSuite
import org.hyperledger.identus.prismtest.utils.TestUtils
import zio.*
import zio.http.Client
import zio.test.*

object MainSpec extends ZIOSpecDefault, TestUtils:

  override def spec =
    val allSpecs =
      CreateOperationSuite.allSpecs +
        UpdateOperationSuite.allSpecs +
        DeactivateOperationSuite.allSpecs

    val prismNodeSpec = suite("PRISM node suite")(allSpecs)
      .provide(NodeClient.grpc("localhost", 50053), NodeName.layer("prism-node"))

    val scalaDidSpec = suite("scala-did node suite")(allSpecs)
      .provide(NodeClient.grpc("localhost", 8980), NodeName.layer("scala-did"))

    val neoprismSpec = suite("NeoPRISM suite")(allSpecs)
      .provide(
        NodeClient.neoprism("localhost", 8080)("localhost", 8090),
        Client.default,
        NodeName.layer("neoprism")
      )

    (neoprismSpec + scalaDidSpec + prismNodeSpec).provide(Runtime.removeDefaultLoggers)
      @@ TestAspect.timed
      @@ TestAspect.withLiveEnvironment
      @@ TestAspect.parallelN(1)
