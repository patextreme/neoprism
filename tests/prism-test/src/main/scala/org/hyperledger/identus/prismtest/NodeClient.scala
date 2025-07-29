package org.hyperledger.identus.prismtest

import io.grpc.ManagedChannelBuilder
import io.iohk.atala.prism.protos.node_api.DIDData
import io.iohk.atala.prism.protos.node_api.GetDidDocumentRequest
import io.iohk.atala.prism.protos.node_api.GetOperationInfoRequest
import io.iohk.atala.prism.protos.node_api.NodeServiceGrpc
import io.iohk.atala.prism.protos.node_api.NodeServiceGrpc.NodeService
import io.iohk.atala.prism.protos.node_api.OperationOutput.OperationMaybe
import io.iohk.atala.prism.protos.node_api.OperationStatus
import io.iohk.atala.prism.protos.node_api.ScheduleOperationsRequest
import org.hyperledger.identus.prismtest.utils.CryptoUtils
import org.hyperledger.identus.prismtest.utils.ProtoUtils
import proto.prism.SignedPrismOperation
import zio.*

import scala.language.implicitConversions

type OperationRef = String

trait NodeClient:
  def scheduleOperations(operations: Seq[SignedPrismOperation]): UIO[Seq[OperationRef]]
  def isOperationConfirmed(ref: OperationRef): UIO[Boolean]
  def getDidDocument(did: String): UIO[Option[DIDData]]

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

  override def getDidDocument(did: String): UIO[Option[DIDData]] =
    ZIO
      .fromFuture(_ => nodeService.getDidDocument(GetDidDocumentRequest(did = did)))
      .orDie
      .map(_.document)
