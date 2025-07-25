import zio.*

object Main extends ZIOAppDefault:
  def run = ZIO.logInfo("hello world")
