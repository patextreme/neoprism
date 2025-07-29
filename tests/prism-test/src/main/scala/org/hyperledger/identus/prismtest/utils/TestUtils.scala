package org.hyperledger.identus.prismtest.utils

import com.google.protobuf.ByteString
import monocle.syntax.all.*
import org.hyperledger.identus.apollo.derivation
import org.hyperledger.identus.apollo.derivation.EdHDKey
import org.hyperledger.identus.apollo.derivation.HDKey
import org.hyperledger.identus.apollo.utils.ByteArrayExtKt
import org.hyperledger.identus.apollo.utils.StringExtKt
import org.hyperledger.identus.prismtest.NodeClient
import org.hyperledger.identus.prismtest.OperationRef
import proto.prism.PrismOperation
import proto.prism.PrismOperation.Operation
import proto.prism.SignedPrismOperation
import proto.prism_ssi.CompressedECKeyData
import proto.prism_ssi.KeyUsage
import proto.prism_ssi.ProtoCreateDID
import proto.prism_ssi.ProtoCreateDID.DIDCreationData
import proto.prism_ssi.PublicKey
import proto.prism_ssi.PublicKey.KeyData
import zio.*

import scala.language.implicitConversions

trait TestUtils extends CryptoUtils, ProtoUtils, TestDsl

trait TestDsl extends ProtoUtils, CryptoUtils:
  def waitUntilConfirmed(operationRefs: Seq[OperationRef]): URIO[NodeClient, Unit] =
    ZIO
      .foreach(operationRefs) { operationRef =>
        ZIO.serviceWithZIO[NodeClient] { nodeClient =>
          nodeClient
            .isOperationConfirmed(operationRef)
            .filterOrFail(identity)(Exception("operation is not confirmed"))
            .debug("isOperationConfirmed")
            .retry(Schedule.recurs(30) && Schedule.spaced(2.seconds))
            .orDie
        }
      }
      .unit

  def builder(seed: Array[Byte]): OpBuilder = OpBuilder(seed)

  extension (ku: KeyUsage)
    def secp256k1(path: String): Array[Byte] => (KeyUsage, HDKey) = (seed: Array[Byte]) =>
      ku -> deriveSecp256k1(seed)(path)
    def ed25519(path: String): Array[Byte] => (KeyUsage, EdHDKey) = (seed: Array[Byte]) =>
      ku -> deriveEd25519(seed)(path)

  extension (op: PrismOperation)
    def signWith(keyId: String, hdKey: HDKey): SignedPrismOperation =
      SignedPrismOperation(
        signedWith = keyId,
        signature = hdKey.getKMMSecp256k1PrivateKey().sign(op.toByteArray),
        operation = Some(op)
      )

  case class OpBuilder(seed: Array[Byte]):
    def createDid: CreateDidOpBuilder = CreateDidOpBuilder(seed, ProtoCreateDID(didData = Some(DIDCreationData())))

  case class CreateDidOpBuilder(seed: Array[Byte], op: ProtoCreateDID):
    def build: PrismOperation = PrismOperation(Operation.CreateDid(op))

    def key(keyId: String)(makeKey: Array[Byte] => (KeyUsage, HDKey | EdHDKey)): CreateDidOpBuilder =
      val (keyUsage, hdKey) = makeKey(seed)
      this
        .focus(_.op.didData.some.publicKeys)
        .modify(_ :+ PublicKey(id = keyId, usage = keyUsage, keyData = hdKey))

trait ProtoUtils:
  given Conversion[Array[Byte], ByteString] = ByteString.copyFrom

  given Conversion[HDKey | EdHDKey, KeyData] = (hdKey: HDKey | EdHDKey) =>
    hdKey match
      case hdKey: HDKey =>
        KeyData.CompressedEcKeyData(
          CompressedECKeyData(
            curve = "secp256k1",
            data = hdKey.getKMMSecp256k1PrivateKey().getPublicKey().getCompressed()
          )
        )
      case hdKey: EdHDKey =>
        KeyData.CompressedEcKeyData(
          CompressedECKeyData(
            curve = "Ed25519",
            data = hdKey.getPrivateKey()
          )
        )

trait CryptoUtils:
  extension (str: String) def decodeHex: Array[Byte] = StringExtKt.decodeHex(str)
  extension (bytes: Array[Byte]) def toHexString: String = ByteArrayExtKt.toHexString(bytes)

  def deriveSecp256k1(seed: Array[Byte])(pathStr: String): HDKey =
    derivation.HDKey(seed, 0, 0).derive(pathStr)

  def deriveEd25519(seed: Array[Byte])(pathStr: String): EdHDKey =
    derivation.EdHDKey.Companion.initFromSeed(seed).derive(pathStr)
