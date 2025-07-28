package org.hyperledger.identus.prismtest

import io.grpc.ManagedChannelBuilder
import org.hyperledger.identus.prismtest.utils.CryptoUtils
import org.hyperledger.identus.prismtest.utils.ProtoUtils
import proto.prism.DIDData
import proto.prism.GetDidDocumentRequest
import proto.prism.GetOperationInfoRequest
import proto.prism.NodeServiceGrpc
import proto.prism.NodeServiceGrpc.NodeService
import proto.prism.OperationOutput.OperationMaybe
import proto.prism.OperationStatus
import proto.prism.ScheduleOperationsRequest
import proto.prism.SignedPrismOperation
import zio.*

import scala.language.implicitConversions

type OperationRef = String

trait NodeClient:
  def scheduleOperations(operations: Seq[SignedPrismOperation]): UIO[Seq[OperationRef]]
  def getDidDocument(did: String): UIO[DIDData]
  def isOperationConfirmed(ref: OperationRef): UIO[Boolean]

object NodeClient:

  def grpc(host: String, port: Int): TaskLayer[NodeClient] =
    ZLayer.fromZIO(
      ZIO
        .attempt(NodeServiceGrpc.stub(ManagedChannelBuilder.forAddress(host, port).usePlaintext.build))
        .map(GrpcNodeClient(_))
    )

private class GrpcNodeClient(nodeService: NodeService) extends NodeClient, CryptoUtils, ProtoUtils:

  override def scheduleOperations(operations: Seq[SignedPrismOperation]): UIO[Seq[OperationRef]] =
    ZIO
      .fromFuture(_ => nodeService.scheduleOperations(ScheduleOperationsRequest(signedOperations = operations)))
      .flatMap(response =>
        ZIO.foreach(response.outputs.map(_.operationMaybe)) {
          case OperationMaybe.OperationId(id) => ZIO.succeed(id.toByteArray().toHexString)
          case _                              => ZIO.dieMessage("operation unsuccessful")
        }
      )
      .orDie

  override def isOperationConfirmed(ref: OperationRef): UIO[Boolean] =
    ZIO
      .fromFuture(_ => nodeService.getOperationInfo(GetOperationInfoRequest(ref.decodeHex)))
      .map(_.operationStatus match
        case OperationStatus.CONFIRMED_AND_APPLIED  => true
        case OperationStatus.CONFIRMED_AND_REJECTED => true
        case _                                      => false)
      .orDie

  override def getDidDocument(did: String): UIO[DIDData] =
    ZIO
      .fromFuture(_ => nodeService.getDidDocument(GetDidDocumentRequest(did = did)))
      .orDie
      .map(_.document)
      .someOrElseZIO(ZIO.dieMessage("DIDData does not exist"))
