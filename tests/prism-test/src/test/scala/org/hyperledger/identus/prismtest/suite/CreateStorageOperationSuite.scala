package org.hyperledger.identus.prismtest.suite

import io.iohk.atala.prism.protos.node_api.DIDData
import org.hyperledger.identus.prismtest.utils.TestUtils
import org.hyperledger.identus.prismtest.NodeName
import proto.prism_ssi.KeyUsage
import zio.test.*
import zio.test.Assertion.*

object CreateStorageOperationSuite extends TestUtils:

  private def extractStorage(didData: DIDData): Seq[String] =
    didData.storageData
      .flatMap(_.data.bytes)
      .map(_.toByteArray().toHexString)

  def allSpecs = suite("CreateStorageOperation")(
    publicKeySpec
  ) @@ NodeName.skipIf("prism-node")

  private def publicKeySpec = suite("PublicKey")(
    test("create storage with valid VDR key should be indexed") {
      for
        seed <- newSeed
        spo1 = builder(seed).createDid
          .key("master-0")(KeyUsage.MASTER_KEY secp256k1 "m/0'/1'/0'")
          .key("vdr-0")(KeyUsage.VDR_KEY secp256k1 "m/0'/8'/0'")
          .build
          .signWith("master-0", deriveSecp256k1(seed)("m/0'/1'/0'"))
        did = spo1.getDid.get
        spo2 = builder(seed)
          .createStorage(spo1.getDid.get)
          .bytes("001122".decodeHex)
          .build
          .signWith("vdr-0", deriveSecp256k1(seed)("m/0'/8'/0'"))
        _ <- scheduleOperations(Seq(spo1, spo2))
        storage <- getDidDocument(did).map(_.get).map(extractStorage)
      yield assert(storage)(hasSameElements(Seq("001122")))
    } @@ TestAspect.tag("dev")
  )
