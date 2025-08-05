package org.hyperledger.identus.prismtest.suite

import org.hyperledger.identus.prismtest.utils.TestUtils
import org.hyperledger.identus.prismtest.NodeName
import proto.prism_ssi.KeyUsage
import zio.test.*
import zio.test.Assertion.*

object UpdateOperationSuite extends TestUtils:
  // TODO: check if scala-did is patched correctly
  def allSpecs = suite("UpdateDidOperation")(
    addPublicKeySpec,
    removePublicKeySpec
  ) @@ NodeName.skipIf("scala-did")

  private def addPublicKeySpec = suite("AddPublicKey action")(
    test("update operation with add-key action should be indexed successfully") {
      for
        seed <- newSeed
        spo1 = builder(seed).createDid
          .key("master-0")(KeyUsage.MASTER_KEY secp256k1 "m/0'/1'/0'")
          .build
          .signWith("master-0", deriveSecp256k1(seed)("m/0'/1'/0'"))
        did = spo1.getDid.get
        spo2 = builder(seed)
          .updateDid(spo1.getOperationHash.get, did)
          .addKey("master-1")(KeyUsage.MASTER_KEY secp256k1 "m/0'/1'/1'")
          .build
          .signWith("master-0", deriveSecp256k1(seed)("m/0'/1'/0'"))
        operationRefs <- scheduleOperations(Seq(spo1, spo2))
        didData <- getDidDocument(did).map(_.get)
      yield assert(didData.publicKeys.map(_.id))(hasSameElements(Seq("master-0", "master-1")))
    },
    test("update operation with add-key action that add the existing key-id should not be indexed") {
      for
        seed <- newSeed
        spo1 = builder(seed).createDid
          .key("master-0")(KeyUsage.MASTER_KEY secp256k1 "m/0'/1'/0'")
          .build
          .signWith("master-0", deriveSecp256k1(seed)("m/0'/1'/0'"))
        did = spo1.getDid.get
        spo2 = builder(seed)
          .updateDid(spo1.getOperationHash.get, did)
          .addKey("master-0")(KeyUsage.MASTER_KEY secp256k1 "m/0'/1'/1'")
          .addKey("master-1")(KeyUsage.MASTER_KEY secp256k1 "m/0'/1'/2'")
          .build
          .signWith("master-0", deriveSecp256k1(seed)("m/0'/1'/0'"))
        operationRefs <- scheduleOperations(Seq(spo1, spo2))
        didData <- getDidDocument(did).map(_.get)
      yield assert(didData.publicKeys.map(_.id))(hasSameElements(Seq("master-0")))
    },
    test("update operation with add-key action that add until 50 keys should be indexed successfully") {
      for
        seed <- newSeed
        spo1 = builder(seed).createDid
          .key("master-0")(KeyUsage.MASTER_KEY secp256k1 "m/0'/1'/0'")
          .build
          .signWith("master-0", deriveSecp256k1(seed)("m/0'/1'/0'"))
        did = spo1.getDid.get
        spo2 = (1 until 50)
          .foldLeft(builder(seed).updateDid(spo1.getOperationHash.get, did)) { case (acc, n) =>
            acc.addKey(s"master-$n")(KeyUsage.MASTER_KEY secp256k1 s"m/0'/1'/$n'")
          }
          .build
          .signWith("master-0", deriveSecp256k1(seed)("m/0'/1'/0'"))
        operationRefs <- scheduleOperations(Seq(spo1, spo2))
        didData <- getDidDocument(did).map(_.get)
      yield assert(didData.publicKeys)(hasSize(equalTo(50)))
    },
    test("update operation with add-key action that add until 51 keys should not be indexed") {
      for
        seed <- newSeed
        spo1 = builder(seed).createDid
          .key("master-0")(KeyUsage.MASTER_KEY secp256k1 "m/0'/1'/0'")
          .build
          .signWith("master-0", deriveSecp256k1(seed)("m/0'/1'/0'"))
        did = spo1.getDid.get
        spo2 = (1 until 51)
          .foldLeft(builder(seed).updateDid(spo1.getOperationHash.get, did)) { case (acc, n) =>
            acc.addKey(s"master-$n")(KeyUsage.MASTER_KEY secp256k1 s"m/0'/1'/$n'")
          }
          .build
          .signWith("master-0", deriveSecp256k1(seed)("m/0'/1'/0'"))
        operationRefs <- scheduleOperations(Seq(spo1, spo2))
        didData <- getDidDocument(did).map(_.get)
      yield assert(didData.publicKeys)(hasSize(equalTo(1)))
    },
    test("update operation with add-key action having key-id of 50 chars should be indexed successfully") {
      for
        seed <- newSeed
        spo1 = builder(seed).createDid
          .key("master-0")(KeyUsage.MASTER_KEY secp256k1 "m/0'/1'/0'")
          .build
          .signWith("master-0", deriveSecp256k1(seed)("m/0'/1'/0'"))
        did = spo1.getDid.get
        spo2 = builder(seed)
          .updateDid(spo1.getOperationHash.get, did)
          .addKey("0" * 50)(KeyUsage.MASTER_KEY secp256k1 s"m/0'/1'/1'")
          .build
          .signWith("master-0", deriveSecp256k1(seed)("m/0'/1'/0'"))
        operationRefs <- scheduleOperations(Seq(spo1, spo2))
        didData <- getDidDocument(did).map(_.get)
      yield assert(didData.publicKeys)(hasSize(equalTo(2)))
    },
    test("update operation with add-key action having key-id of 51 chars should not be indexed") {
      for
        seed <- newSeed
        spo1 = builder(seed).createDid
          .key("master-0")(KeyUsage.MASTER_KEY secp256k1 "m/0'/1'/0'")
          .build
          .signWith("master-0", deriveSecp256k1(seed)("m/0'/1'/0'"))
        did = spo1.getDid.get
        spo2 = builder(seed)
          .updateDid(spo1.getOperationHash.get, did)
          .addKey("0" * 51)(KeyUsage.MASTER_KEY secp256k1 s"m/0'/1'/1'")
          .build
          .signWith("master-0", deriveSecp256k1(seed)("m/0'/1'/0'"))
        operationRefs <- scheduleOperations(Seq(spo1, spo2), batch = false)
        didData <- getDidDocument(did).map(_.get)
      yield assert(didData.publicKeys)(hasSize(equalTo(1)))
    }
  )

  private def removePublicKeySpec = suite("RemovePublicKey action")(
    test("update operation with remove-key action should be indexed successfully") {
      for
        seed <- newSeed
        spo1 = builder(seed).createDid
          .key("master-0")(KeyUsage.MASTER_KEY secp256k1 "m/0'/1'/0'")
          .key("master-1")(KeyUsage.MASTER_KEY secp256k1 "m/0'/1'/0'")
          .build
          .signWith("master-0", deriveSecp256k1(seed)("m/0'/1'/0'"))
        did = spo1.getDid.get
        spo2 = builder(seed)
          .updateDid(spo1.getOperationHash.get, did)
          .removeKey("master-1")
          .build
          .signWith("master-0", deriveSecp256k1(seed)("m/0'/1'/0'"))
        operationRefs <- scheduleOperations(Seq(spo1, spo2))
        didData <- getDidDocument(did).map(_.get)
      yield assert(didData.publicKeys.map(_.id))(hasSameElements(Seq("master-0")))
    },
    test("update operation with remove-key action that remove all master-key should not be indexed") {
      for
        seed <- newSeed
        spo1 = builder(seed).createDid
          .key("master-0")(KeyUsage.MASTER_KEY secp256k1 "m/0'/1'/0'")
          .build
          .signWith("master-0", deriveSecp256k1(seed)("m/0'/1'/0'"))
        did = spo1.getDid.get
        spo2 = builder(seed)
          .updateDid(spo1.getOperationHash.get, did)
          .removeKey("master-0")
          .build
          .signWith("master-0", deriveSecp256k1(seed)("m/0'/1'/0'"))
        operationRefs <- scheduleOperations(Seq(spo1, spo2))
        didData <- getDidDocument(did).map(_.get)
      yield assert(didData.publicKeys.map(_.id))(hasSameElements(Seq("master-0")))
    }
  )
