package org.hyperledger.identus.prismtest

import zio.*
import zio.test.*

case class NodeName(name: String)

object NodeName:
  def layer(name: String): ULayer[NodeName] = ZLayer.succeed(NodeName(name))

  def skipIf(name: String): TestAspect[Nothing, NodeName, Nothing, Any] =
    new TestAspect[Nothing, NodeName, Nothing, Any]:
      def some[R <: NodeName, E](spec: Spec[R, E])(implicit trace: Trace): Spec[R, E] =
        spec.whenZIO[R, E](ZIO.serviceWith[NodeName](_.name != name))
