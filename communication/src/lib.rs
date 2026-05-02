#[tarpc::service]
pub trait Communication {
    async fn hello(name: String) -> String;
}
