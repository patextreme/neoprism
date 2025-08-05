package org.hyperledger.identus.prismtest.suite

import org.hyperledger.identus.prismtest.utils.TestUtils
import org.hyperledger.identus.prismtest.NodeName
import proto.prism_ssi.KeyUsage
import zio.test.*
import zio.test.Assertion.*
import zio.ZIO

object CreateOperationSuite extends TestUtils:
  // TODO: add tests for context
  def allSpecs = suite("CreateDidOperation")(publicKeySpec, serviceSpec, vdrSpec)

  private def publicKeySpec = suite("PublicKey")(
    test("create operation with only master-key should be indexed successfully") {
      for
        seed <- newSeed
        spo = builder(seed).createDid
          .key("master-0")(KeyUsage.MASTER_KEY secp256k1 "m/0'/1'/0'")
          .build
          .signWith("master-0", deriveSecp256k1(seed)("m/0'/1'/0'"))
        operationRefs <- scheduleOperations(Seq(spo))
        didData <- getDidDocument(spo.getDid.get).map(_.get)
      yield assert(didData.context)(isEmpty) &&
        assert(didData.services)(isEmpty) &&
        assert(didData.publicKeys)(hasSize(equalTo(1)))
    },
    test("create operation with all key types should be indexed successfully") {
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
        didData <- getDidDocument(spo.getDid.get).map(_.get)
      yield assert(didData.context)(isEmpty) &&
        assert(didData.services)(isEmpty) &&
        assert(didData.publicKeys)(hasSize(equalTo(8))) &&
        assert(didData.publicKeys.map(_.usage).distinct)(hasSize(equalTo(8)))
    } @@ NodeName.skipIf("prism-node"),
    test("create operation with non-secp256k1 master-key should not be indexed") {
      for
        seed <- newSeed
        spo = builder(seed).createDid
          .key("master-0")(KeyUsage.MASTER_KEY ed25519 "m/0'/1'/0'")
          .build
          .signWith("master-0", deriveSecp256k1(seed)("m/0'/1'/0'"))
        operationRefs <- scheduleOperations(Seq(spo))
        didData <- getDidDocument(spo.getDid.get)
      yield assert(didData)(isNone)
    },
    test("create operation without master-key should not be be indexed") {
      for
        seed <- newSeed
        spo = builder(seed).createDid
          .key("auth-0")(KeyUsage.AUTHENTICATION_KEY secp256k1 "m/0'/4'/0'")
          .build
          .signWith("auth-0", deriveSecp256k1(seed)("m/0'/4'/0'"))
        operationRefs <- scheduleOperations(Seq(spo))
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
        didData1 <- getDidDocument(spo1.getDid.get)
        didData2 <- getDidDocument(spo2.getDid.get)
      yield assert(didData1)(isNone) && assert(didData2)(isNone)
    },
    test("create operation with 50 keys should be indexed successfully") {
      for
        seed <- newSeed
        spo = (0 until 50)
          .foldLeft(builder(seed).createDid) { case (acc, n) =>
            acc.key(s"master-$n")(KeyUsage.MASTER_KEY secp256k1 s"m/0'/1'/$n'")
          }
          .build
          .signWith("master-0", deriveSecp256k1(seed)("m/0'/1'/0'"))
        operationRefs <- scheduleOperations(Seq(spo))
        didData <- getDidDocument(spo.getDid.get).map(_.get)
      yield assert(didData.publicKeys.length)(equalTo(50))
    },
    test("create operation with 51 keys should not be indexed") {
      for
        seed <- newSeed
        spo = (0 until 51)
          .foldLeft(builder(seed).createDid) { case (acc, n) =>
            acc.key(s"master-$n")(KeyUsage.MASTER_KEY secp256k1 s"m/0'/1'/$n'")
          }
          .build
          .signWith("master-0", deriveSecp256k1(seed)("m/0'/1'/0'"))
        operationRefs <- scheduleOperations(Seq(spo))
        didData <- getDidDocument(spo.getDid.get)
      yield assert(didData)(isNone)
    } @@ NodeName.skipIf("scala-did"),
    test("create operation with key-id of 50 chars should be indexed successfully") {
      for
        seed <- newSeed
        spo = builder(seed).createDid
          .key("0" * 50)(KeyUsage.MASTER_KEY secp256k1 "m/0'/1'/0'")
          .build
          .signWith("0" * 50, deriveSecp256k1(seed)("m/0'/1'/0'"))
        operationRefs <- scheduleOperations(Seq(spo))
        didData <- getDidDocument(spo.getDid.get).map(_.get)
      yield assert(didData.publicKeys)(hasSize(equalTo(1)))
    },
    test("create operation with key-id of 51 chars should not be indexed") {
      for
        seed <- newSeed
        spo = builder(seed).createDid
          .key("0" * 51)(KeyUsage.MASTER_KEY secp256k1 "m/0'/1'/0'")
          .build
          .signWith("0" * 51, deriveSecp256k1(seed)("m/0'/1'/0'"))
        operationRefs <- scheduleOperations(Seq(spo))
        didData <- getDidDocument(spo.getDid.get)
      yield assert(didData)(isNone)
    } @@ NodeName.skipIf("scala-did"),
    test("create operation with key-id not a URL fragment should not be indexed") {
      for
        seed <- newSeed
        spo = builder(seed).createDid
          .key("master 0")(KeyUsage.MASTER_KEY secp256k1 "m/0'/1'/0'")
          .build
          .signWith("master 0", deriveSecp256k1(seed)("m/0'/1'/0'"))
        operationRefs <- scheduleOperations(Seq(spo))
        didData <- getDidDocument(spo.getDid.get)
      yield assert(didData)(isNone)
    } @@ NodeName.skipIf("scala-did")
  )

  private def serviceSpec = suite("Service")(
    test("create operation with 50 services should be indexed successfully") {
      for
        seed <- newSeed
        opBuider = builder(seed).createDid
          .key(s"master-0")(KeyUsage.MASTER_KEY secp256k1 s"m/0'/1'/0'")
        spo = (0 until 50)
          .foldLeft(opBuider) { case (acc, n) => acc.service(s"service-$n")("LinkedDomains", "https://example.com") }
          .build
          .signWith("master-0", deriveSecp256k1(seed)("m/0'/1'/0'"))
        operationRefs <- scheduleOperations(Seq(spo))
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
        didData <- getDidDocument(spo.getDid.get)
      yield assert(didData)(isNone)
    } @@ NodeName.skipIf("scala-did"),
    test("create operation with service-id of 50 chars should be indexed successfully") {
      for
        seed <- newSeed
        spo = builder(seed).createDid
          .key("master-0")(KeyUsage.MASTER_KEY secp256k1 "m/0'/1'/0'")
          .service("0" * 50)("LinkedDomains", "https://example.com")
          .build
          .signWith("master-0", deriveSecp256k1(seed)("m/0'/1'/0'"))
        operationRefs <- scheduleOperations(Seq(spo))
        didData <- getDidDocument(spo.getDid.get).map(_.get)
      yield assert(didData.services)(hasSize(equalTo(1)))
    },
    test("create operation with service-id of 51 chars should not be indexed") {
      for
        seed <- newSeed
        spo = builder(seed).createDid
          .key("master-0")(KeyUsage.MASTER_KEY secp256k1 "m/0'/1'/0'")
          .service("0" * 51)("LinkedDomains", "https://example.com")
          .build
          .signWith("master-0", deriveSecp256k1(seed)("m/0'/1'/0'"))
        operationRefs <- scheduleOperations(Seq(spo))
        didData <- getDidDocument(spo.getDid.get)
      yield assert(didData)(isNone)
    } @@ NodeName.skipIf("scala-did"),
    test("create operation with service-id not a URL fragment should not be indexed") {
      for
        seed <- newSeed
        spo = builder(seed).createDid
          .key("master-0")(KeyUsage.MASTER_KEY secp256k1 "m/0'/1'/0'")
          .service("service 0")("LinkedDomains", "https://example.com")
          .build
          .signWith("master-0", deriveSecp256k1(seed)("m/0'/1'/0'"))
        operationRefs <- scheduleOperations(Seq(spo))
        didData <- getDidDocument(spo.getDid.get)
      yield assert(didData)(isNone)
    } @@ NodeName.skipIf("scala-did"),
    test("create operation with service-type of 100 chars should be indexed successfully") {
      for
        seed <- newSeed
        spo = builder(seed).createDid
          .key("master-0")(KeyUsage.MASTER_KEY secp256k1 "m/0'/1'/0'")
          .service("service-0")("0" * 100, "https://example.com")
          .build
          .signWith("master-0", deriveSecp256k1(seed)("m/0'/1'/0'"))
        operationRefs <- scheduleOperations(Seq(spo))
        didData <- getDidDocument(spo.getDid.get).map(_.get)
      yield assert(didData.services)(hasSize(equalTo(1)))
    },
    test("create operation with service-type of 101 chars should not be indexed") {
      for
        seed <- newSeed
        spo = builder(seed).createDid
          .key("master-0")(KeyUsage.MASTER_KEY secp256k1 "m/0'/1'/0'")
          .service("service-0")("0" * 101, "https://example.com")
          .build
          .signWith("master-0", deriveSecp256k1(seed)("m/0'/1'/0'"))
        operationRefs <- scheduleOperations(Seq(spo))
        didData <- getDidDocument(spo.getDid.get)
      yield assert(didData)(isNone)
    } @@ NodeName.skipIf("scala-did"),
    test("create operation with service-type not following ABNF should not be indexed") {
      for
        seed <- newSeed
        buildOperation = (serviceType: String) =>
          builder(seed).createDid
            .key("master-0")(KeyUsage.MASTER_KEY secp256k1 "m/0'/1'/0'")
            .service("service-0")(serviceType, "https://example.com")
            .build
            .signWith("master-0", deriveSecp256k1(seed)("m/0'/1'/0'"))
        spos = Seq(
          buildOperation(""),
          buildOperation(" "),
          buildOperation(" LinkedDomains"),
          buildOperation("LinkedDomains "),
          buildOperation("Linked@Domains"),
          buildOperation("[]"),
          buildOperation("[LinkedDomains ]"),
          buildOperation("[\" LinkedDomains\"]"),
          buildOperation("[\"LinkedDomains \"]"),
          buildOperation("[\"Linked@Domains\"]")
        )
        operationRefs <- scheduleOperations(spos)
        didDataList <- ZIO.foreach(spos) { spo => getDidDocument(spo.getDid.get) }
      yield assert(didDataList)(forall(isNone))
    } @@ NodeName.skipIf("scala-did"),
    test("create operation with service-endpoint of 300 chars should be indexed successfully") {
      for
        seed <- newSeed
        serviceEndpoint = s"http://example.com/${"0" * 300}".take(300)
        spo = builder(seed).createDid
          .key("master-0")(KeyUsage.MASTER_KEY secp256k1 "m/0'/1'/0'")
          .service("service-0")("LinkedDomais", serviceEndpoint)
          .build
          .signWith("master-0", deriveSecp256k1(seed)("m/0'/1'/0'"))
        operationRefs <- scheduleOperations(Seq(spo))
        didData <- getDidDocument(spo.getDid.get).map(_.get)
      yield assert(didData.services)(hasSize(equalTo(1)))
    },
    test("create operation with service-endpoint of 301 chars should not be indexed") {
      for
        seed <- newSeed
        serviceEndpoint = s"http://example.com/${"0" * 300}".take(301)
        spo = builder(seed).createDid
          .key("master-0")(KeyUsage.MASTER_KEY secp256k1 "m/0'/1'/0'")
          .service("service-0")("LinkedDomais", serviceEndpoint)
          .build
          .signWith("master-0", deriveSecp256k1(seed)("m/0'/1'/0'"))
        operationRefs <- scheduleOperations(Seq(spo))
        didData <- getDidDocument(spo.getDid.get)
      yield assert(didData)(isNone)
    } @@ NodeName.skipIf("scala-did"),
    test("create operation with service-endpoint not following ABNF should not be indexed") {
      for
        seed <- newSeed
        buildOperation = (serviceEndpoint: String) =>
          builder(seed).createDid
            .key("master-0")(KeyUsage.MASTER_KEY secp256k1 "m/0'/1'/0'")
            .service("service-0")("LinkedDomains", serviceEndpoint)
            .build
            .signWith("master-0", deriveSecp256k1(seed)("m/0'/1'/0'"))
        spos = Seq(
          buildOperation(""),
          buildOperation(" "),
          buildOperation(" http://example.com"),
          buildOperation("http://example.com "),
          buildOperation("not a url"),
          buildOperation("[\"not a url\"]"),
          buildOperation("[\" http://example.com\"]"),
          buildOperation("[\"http://example.com \"]"),
          buildOperation("123")
        )
        operationRefs <- scheduleOperations(spos)
        didDataList <- ZIO.foreach(spos) { spo => getDidDocument(spo.getDid.get) }
      yield assert(didDataList)(forall(isNone))
    } @@ NodeName.skipIf("scala-did")
  )

  private def vdrSpec = suite("VDR")(
    test("create operation with non-secp256k1 vdr key should not be indexed") {
      for
        seed <- newSeed
        spo = builder(seed).createDid
          .key("master-0")(KeyUsage.MASTER_KEY secp256k1 "m/0'/1'/0'")
          .key("vdr-0")(KeyUsage.VDR_KEY ed25519 "m/0'/8'/0'")
          .build
          .signWith("master-0", deriveSecp256k1(seed)("m/0'/1'/0'"))
        operationRefs <- scheduleOperations(Seq(spo))
        didData <- getDidDocument(spo.getDid.get)
      yield assert(didData)(isNone)
    } @@ NodeName.skipIf("prism-node", "scala-did")
  )
