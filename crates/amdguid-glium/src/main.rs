mod backend;

#[tokio::main]
async fn main() {
    amdguid::start_app(|amd_gui, receiver| {
        backend::run_app(amd_gui, receiver);
    })
    .await;
}
