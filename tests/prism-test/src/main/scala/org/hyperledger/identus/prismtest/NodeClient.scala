package org.hyperledger.identus.prismtest

import io.grpc.ManagedChannelBuilder
import proto.prism.DIDData
import proto.prism.GetDidDocumentRequest
import proto.prism.NodeServiceGrpc
import proto.prism.NodeServiceGrpc.NodeService
import proto.prism.ScheduleOperationsRequest
import proto.prism.SignedPrismOperation
import zio.*

trait NodeClient:
  def scheduleOperations(operations: Seq[SignedPrismOperation]): UIO[Unit]
  def getDidDocument(did: String): UIO[DIDData]

object NodeClient:

  def grpc(host: String, port: Int): TaskLayer[NodeClient] =
    ZLayer.fromZIO(
      ZIO
        .attempt(NodeServiceGrpc.stub(ManagedChannelBuilder.forAddress(host, port).usePlaintext.build))
        .map(GrpcNodeClient(_))
    )

private class GrpcNodeClient(nodeService: NodeService) extends NodeClient:

  override def scheduleOperations(operations: Seq[SignedPrismOperation]): UIO[Unit] =
    ZIO
      .fromFuture(_ => nodeService.scheduleOperations(ScheduleOperationsRequest(signedOperations = operations)))
      .orDie
      .unit

  override def getDidDocument(did: String): UIO[DIDData] =
    ZIO
      .fromFuture(_ => nodeService.getDidDocument(GetDidDocumentRequest(did = did)))
      .orDie
      .map(_.document)
      .someOrElseZIO(ZIO.dieMessage("DIDData does not exist"))
