use prism_node::build_rocket;
use rocket::launch;

#[launch]
fn rocket() -> _ {
    build_rocket()
}
