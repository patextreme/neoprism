use prism_persistence::db::PrismDB;

#[tokio::main]
async fn main() {
    let db_url = "sqlite://target/tmp.db"; // TODO: do not hard code this
    let prism_db = PrismDB::from_url(db_url).await.unwrap();
    prism_db.migrate().await.unwrap();
}
