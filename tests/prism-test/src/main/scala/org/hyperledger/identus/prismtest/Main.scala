package org.hyperledger.identus.prismtest

import org.hyperledger.identus.prismtest.utils.TestUtils
import proto.prism_ssi.KeyUsage
import zio.*

object Main extends ZIOAppDefault, TestUtils:

  // MASTER_KEY = 1;
  // ISSUING_KEY = 2;
  // KEY_AGREEMENT_KEY = 3;
  // AUTHENTICATION_KEY = 4;
  // REVOCATION_KEY = 5;
  // CAPABILITY_INVOCATION_KEY = 6;
  // CAPABILITY_DELEGATION_KEY = 7;
  // VDR_KEY = 8;

  val SEED: Array[Byte] = Array.fill[Byte](64)(0)

  override def run =
    val spo = builder(SEED).createDid
      .key("master-0")(KeyUsage.MASTER_KEY secp256k1 "m/0'/1'/0'")
      .key("vdr-0")(KeyUsage.VDR_KEY secp256k1 "m/0'/1'/0'")
      .build
      .signWith("master-0", deriveSecp256k1(SEED)("m/0'/1'/0'"))

    ZIO.debug(spo.toByteArray.toHexString)
