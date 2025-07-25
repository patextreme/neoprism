import proto.prism.PrismBlock
import proto.prism.SignedPrismOperation
import zio.*

object Main extends ZIOAppDefault:
  override def run = ZIO.logInfo("hello world")
