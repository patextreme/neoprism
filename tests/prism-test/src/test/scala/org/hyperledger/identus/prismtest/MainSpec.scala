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
    createOperationSuite
      .provide(NodeClient.grpc("localhost", 50053))
      @@ TestAspect.withLiveClock
      @@ TestAspect.withLiveRandom

  private def createOperationSuite = suite("CreateDidOperation spec")(
    test("create operation with only master key") {
      for
        client <- ZIO.service[NodeClient]
        seed <- newSeed
        spo = builder(seed).createDid
          .key("master-0")(KeyUsage.MASTER_KEY secp256k1 "m/0'/1'/0'")
          .build
          .signWith("master-0", deriveSecp256k1(seed)("m/0'/1'/0'"))
        operationRefs <- client.scheduleOperations(Seq(spo))
        _ <- waitUntilConfirmed(operationRefs)
        didData <- client.getDidDocument(spo.getDid.get).map(_.get)
      yield assert(didData.context)(isEmpty) &&
        assert(didData.services)(isEmpty) &&
        assert(didData.publicKeys)(hasSize(equalTo(1)))
    },
    test("create operation with signedWith key not found") {
      for
        client <- ZIO.service[NodeClient]
        seed <- newSeed
        spo = builder(seed).createDid
          .key("master-0")(KeyUsage.MASTER_KEY secp256k1 "m/0'/1'/0'")
          .build
          .signWith("master-1", deriveSecp256k1(seed)("m/0'/1'/0'"))
        operationRefs <- client.scheduleOperations(Seq(spo))
        _ <- waitUntilConfirmed(operationRefs)
        didData <- client.getDidDocument(spo.getDid.get)
      yield assert(didData)(isNone)
    }
  )
