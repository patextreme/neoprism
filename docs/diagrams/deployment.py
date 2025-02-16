from diagrams import Diagram, Cluster, Edge
from diagrams.aws.general import User
from diagrams.custom import Custom
from diagrams.onprem.database import Postgresql
from diagrams.onprem.network import Nginx
from diagrams.programming.language import Bash, Rust


def cardano_node(name: str):
    return Custom(name, "./resources/cardano.png")


def deploy_standalone():
    with Diagram(
        "Standalone mode", show=False, filename="deploy_standalone", direction="TB"
    ):
        with Cluster("Internet"):
            cardano = cardano_node("Cardano node")

        with Cluster("On-prem"):
            neoprism = Rust("NeoPRISM\n(standalone)")
            database = Postgresql()

        user = User("Users")
        client = Bash("API clients")

        user >> Edge(label="WebUI") >> neoprism
        client >> Edge(label="Resolver API") >> neoprism
        neoprism >> cardano
        neoprism >> database


def deploy_ha():
    with Diagram(
        "High-availability mode", show=False, filename="deploy_ha", direction="TB"
    ):
        with Cluster("Internet"):
            cardano = cardano_node("Cardano node")

        with Cluster("On-prem"):
            reverse_proxy = Nginx()
            neoprism_server_1 = Rust("NeoPRISM\n(server)")
            neoprism_server_2 = Rust("NeoPRISM\n(server)")
            neoprism_server_3 = Rust("NeoPRISM\n(server)")
            neoprism_worker = Rust("NeoPRISM\n(worker)")
            database = Postgresql()

        user = User("Users")
        client = Bash("API clients")

        servers = [neoprism_server_1, neoprism_server_2, neoprism_server_3]

        user >> Edge(label="WebUI") >> reverse_proxy
        client >> Edge(label="Resolver API") >> reverse_proxy
        reverse_proxy >> servers
        neoprism_worker >> cardano
        neoprism_worker >> database
        servers >> database


if __name__ == "__main__":
    deploy_standalone()
    deploy_ha()
