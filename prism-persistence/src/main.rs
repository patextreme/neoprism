use prism_persistence::db::PrismDB;

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    let db_url = args
        .into_iter()
        .take(2)
        .last()
        .expect("Expect an argument to be database url");
    println!("Using db_url: '{}'", db_url);
    let prism_db = PrismDB::from_url(&db_url).await.unwrap();
    prism_db.migrate().await.unwrap();
}
