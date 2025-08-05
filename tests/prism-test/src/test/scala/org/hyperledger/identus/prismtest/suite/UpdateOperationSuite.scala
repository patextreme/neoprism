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
    test("update operation with add-key action is indexed successfully") {
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
        _ <- waitUntilConfirmed(operationRefs)
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
        _ <- waitUntilConfirmed(operationRefs)
        didData <- getDidDocument(did).map(_.get)
      yield assert(didData.publicKeys.map(_.id))(hasSameElements(Seq("master-0")))
    }
  )

  private def removePublicKeySpec = suite("RemovePublicKey action")(
    test("update operation with remove-key action is indexed successfully") {
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
        _ <- waitUntilConfirmed(operationRefs)
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
        _ <- waitUntilConfirmed(operationRefs)
        didData <- getDidDocument(did).map(_.get)
      yield assert(didData.publicKeys.map(_.id))(hasSameElements(Seq("master-0")))
    }
  )
