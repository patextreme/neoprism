package org.hyperledger.identus.prismtest.suite

import org.hyperledger.identus.prismtest.utils.TestUtils
import org.hyperledger.identus.prismtest.NodeName
import proto.prism_ssi.KeyUsage
import zio.test.*
import zio.test.Assertion.*

object CreateOperationSuite extends TestUtils:
  def values = suite("CreateDidOperation spec")(
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
    test("create operation with all other keys") {
      for
        seed <- newSeed
        spo = builder(seed).createDid
          .key("master-0")(KeyUsage.MASTER_KEY secp256k1 "m/0'/1'/0'")
          .key("iss-0")(KeyUsage.ISSUING_KEY ed25519 "m/0'/2'/0'")
          .key("comm-0")(KeyUsage.KEY_AGREEMENT_KEY secp256k1 "m/0'/3'/0'")
          .key("auth-0")(KeyUsage.AUTHENTICATION_KEY ed25519 "m/0'/4'/0'")
          .key("revoke-0")(KeyUsage.REVOCATION_KEY secp256k1 "m/0'/5'/0'")
          .key("capinv-0")(KeyUsage.CAPABILITY_INVOCATION_KEY secp256k1 "m/0'/6'/0'")
          .key("capdel-0")(KeyUsage.CAPABILITY_DELEGATION_KEY secp256k1 "m/0'/7'/0'")
          .build
          .signWith("master-0", deriveSecp256k1(seed)("m/0'/1'/0'"))
        operationRefs <- scheduleOperations(Seq(spo))
        _ <- waitUntilConfirmed(operationRefs)
        didData <- getDidDocument(spo.getDid.get).map(_.get)
      yield assert(didData.context)(isEmpty) &&
        assert(didData.services)(isEmpty) &&
        assert(didData.publicKeys)(hasSize(equalTo(7)))
    },
    test("create operation with master key that is not secp256k1") {
      for
        seed <- newSeed
        spo = builder(seed).createDid
          .key("master-0")(KeyUsage.MASTER_KEY ed25519 "m/0'/1'/0'")
          .build
          .signWith("master-0", deriveSecp256k1(seed)("m/0'/1'/0'"))
        operationRefs <- scheduleOperations(Seq(spo))
        _ <- waitUntilConfirmed(operationRefs)
        didData <- getDidDocument(spo.getDid.get)
      yield assert(didData)(isNone)
    },
    test("create operation with non-master signedWith key") {
      for
        seed <- newSeed
        spo = builder(seed).createDid
          .key("master-0")(KeyUsage.AUTHENTICATION_KEY secp256k1 "m/0'/4'/0'")
          .build
          .signWith("master-0", deriveSecp256k1(seed)("m/0'/4'/0'"))
        operationRefs <- scheduleOperations(Seq(spo))
        _ <- waitUntilConfirmed(operationRefs)
        didData <- getDidDocument(spo.getDid.get)
      yield assert(didData)(isNone)
    } @@ NodeName.skipIf("prism-node"),
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
