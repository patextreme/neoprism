package org.hyperledger.identus.prismtest.suite

import org.hyperledger.identus.prismtest.utils.TestUtils
import org.hyperledger.identus.prismtest.NodeName
import proto.prism_ssi.KeyUsage
import zio.test.*
import zio.test.Assertion.*

object CreateOperationSuite extends TestUtils:
  def allSpecs = suite("CreateDidOperation specs")(publicKeySpecs, serviceSpecs)

  private def publicKeySpecs = suite("PublicKey specs")(
    test("create operation with only master key is indexed successfully") {
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
    test("create operation with all key types is indexed successfully") {
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
          .key("vdr-0")(KeyUsage.VDR_KEY secp256k1 "m/0'/8'/0'")
          .build
          .signWith("master-0", deriveSecp256k1(seed)("m/0'/1'/0'"))
        operationRefs <- scheduleOperations(Seq(spo))
        _ <- waitUntilConfirmed(operationRefs)
        didData <- getDidDocument(spo.getDid.get).map(_.get)
      yield assert(didData.context)(isEmpty) &&
        assert(didData.services)(isEmpty) &&
        assert(didData.publicKeys)(hasSize(equalTo(8))) &&
        assert(didData.publicKeys.map(_.usage).distinct)(hasSize(equalTo(8)))
    } @@ NodeName.skipIf("prism-node"),
    test("create operation with non-secp256k1 master key should not be indexed") {
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
    test("create operation without master key should not be be indexed") {
      for
        seed <- newSeed
        spo = builder(seed).createDid
          .key("auth-0")(KeyUsage.AUTHENTICATION_KEY secp256k1 "m/0'/4'/0'")
          .build
          .signWith("auth-0", deriveSecp256k1(seed)("m/0'/4'/0'"))
        operationRefs <- scheduleOperations(Seq(spo))
        _ <- waitUntilConfirmed(operationRefs)
        didData <- getDidDocument(spo.getDid.get)
      yield assert(didData)(isNone)
    },
    test("create operation with invalid signedWith key should not be indexed") {
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
    },
    test("create operation with 50 public keys is indexed successfully") {
      for
        seed <- newSeed
        spo = (0 until 50)
          .foldLeft(builder(seed).createDid) { case (acc, n) =>
            acc.key(s"master-$n")(KeyUsage.MASTER_KEY secp256k1 s"m/0'/1'/$n'")
          }
          .build
          .signWith("master-0", deriveSecp256k1(seed)("m/0'/1'/0'"))
        operationRefs <- scheduleOperations(Seq(spo))
        _ <- waitUntilConfirmed(operationRefs)
        didData <- getDidDocument(spo.getDid.get).map(_.get)
      yield assert(didData.publicKeys.length)(equalTo(50))
    },
    test("create operation with 51 public keys should not be indexed") {
      for
        seed <- newSeed
        spo = (0 until 51)
          .foldLeft(builder(seed).createDid) { case (acc, n) =>
            acc.key(s"master-$n")(KeyUsage.MASTER_KEY secp256k1 s"m/0'/1'/$n'")
          }
          .build
          .signWith("master-0", deriveSecp256k1(seed)("m/0'/1'/0'"))
        operationRefs <- scheduleOperations(Seq(spo))
        _ <- waitUntilConfirmed(operationRefs)
        didData <- getDidDocument(spo.getDid.get)
      yield assert(didData)(isNone)
    } @@ NodeName.skipIf("scala-did")
  )

  private def serviceSpecs = suite("Service specs")(
    test("create operation with 50 services is indexed successfully") {
      for
        seed <- newSeed
        opBuider = builder(seed).createDid
          .key(s"master-0")(KeyUsage.MASTER_KEY secp256k1 s"m/0'/1'/0'")
        spo = (0 until 50)
          .foldLeft(opBuider) { case (acc, n) => acc.service(s"service-$n")("LinkedDomains", "https://example.com") }
          .build
          .signWith("master-0", deriveSecp256k1(seed)("m/0'/1'/0'"))
        operationRefs <- scheduleOperations(Seq(spo))
        _ <- waitUntilConfirmed(operationRefs)
        didData <- getDidDocument(spo.getDid.get).map(_.get)
      yield assert(didData.services.length)(equalTo(50))
    },
    test("create operation with 51 services should not be indexed") {
      for
        seed <- newSeed
        opBuider = builder(seed).createDid
          .key(s"master-0")(KeyUsage.MASTER_KEY secp256k1 s"m/0'/1'/0'")
        spo = (0 until 51)
          .foldLeft(opBuider) { case (acc, n) => acc.service(s"service-$n")("LinkedDomains", "https://example.com") }
          .build
          .signWith("master-0", deriveSecp256k1(seed)("m/0'/1'/0'"))
        operationRefs <- scheduleOperations(Seq(spo))
        _ <- waitUntilConfirmed(operationRefs)
        didData <- getDidDocument(spo.getDid.get)
      yield assert(didData)(isNone)
    } @@ NodeName.skipIf("scala-did")
  )
