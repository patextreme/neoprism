package org.hyperledger.identus.prismtest.suite

import io.iohk.atala.prism.protos.node_api.DIDData
import org.hyperledger.identus.prismtest.utils.TestUtils

trait StorageTestUtils extends TestUtils:
  protected def extractStorageHex(didData: DIDData): Seq[String] =
    didData.storageData
      .flatMap(_.data.bytes)
      .map(_.toByteArray().toHexString)
