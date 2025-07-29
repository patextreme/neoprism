package org.hyperledger.identus.prismtest

import org.hyperledger.identus.prismtest.utils.TestUtils
import proto.prism_ssi.KeyUsage
import zio.*
import zio.test.*
import zio.test.Assertion.*

object MainSpec extends ZIOSpecDefault, TestUtils:

  // MASTER_KEY = 1;
  // ISSUING_KEY = 2;
  // KEY_AGREEMENT_KEY = 3;
  // AUTHENTICATION_KEY = 4;
  // REVOCATION_KEY = 5;
  // CAPABILITY_INVOCATION_KEY = 6;
  // CAPABILITY_DELEGATION_KEY = 7;
  // VDR_KEY = 8;

  override def spec =
    (createOperationSuite
      @@ TestAspect.withLiveClock
      @@ TestAspect.withLiveRandom
      @@ TestAspect.timed)
      .provide(NodeClient.grpc("localhost", 50053))
      .provide(Runtime.removeDefaultLoggers)

  private def createOperationSuite = suite("CreateDidOperation spec")(
    test("create operation with only master key") {
      for
        seed <- newSeed
        spo = builder(seed).createDid
          .key("master-0")(KeyUsage.MASTER_KEY secp256k1 "m/0'/1'/0'")
          .build
          .signWith("master-0", deriveSecp256k1(seed)("m/0'/1'/0'"))
        operationRefs <- scheduleOperations(Seq(spo))
        _ <- waitUntilConfirmed(operationRefs)
        didData <- getDidDocument(spo.getDid.get).map(_.get)
      yield assert(didData.context)(isEmpty) &&
        assert(didData.services)(isEmpty) &&
        assert(didData.publicKeys)(hasSize(equalTo(1)))
    },
    test("create operation with invalid signedWith key") {
      for
        seed <- newSeed
        // key id not exist
        spo1 = builder(seed).createDid
          .key("master-0")(KeyUsage.MASTER_KEY secp256k1 "m/0'/1'/0'")
          .build
          .signWith("master-1", deriveSecp256k1(seed)("m/0'/1'/0'"))
        // same key id with wrong private key
        spo2 = builder(seed).createDid
          .key("master-0")(KeyUsage.MASTER_KEY secp256k1 "m/1'/1'/0'")
          .build
          .signWith("master-0", deriveSecp256k1(seed)("m/1'/1'/1'"))
        operationRefs <- scheduleOperations(Seq(spo1, spo2))
        _ <- waitUntilConfirmed(operationRefs)
        didData1 <- getDidDocument(spo1.getDid.get)
        didData2 <- getDidDocument(spo2.getDid.get)
      yield assert(didData1)(isNone) && assert(didData2)(isNone)
    }
  )
