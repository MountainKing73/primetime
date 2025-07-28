use primetime::run;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    run(String::from("0.0.0.0:8080")).await
}
