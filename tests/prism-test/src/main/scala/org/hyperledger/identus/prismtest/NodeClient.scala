package org.hyperledger.identus.prismtest

import proto.prism.SignedPrismOperation
import zio.*

trait NodeClient:
  def scheduleOperations(operations: Seq[SignedPrismOperation]): UIO[Unit]
