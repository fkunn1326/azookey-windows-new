use protos::proto::azookey_service_client::AzookeyServiceClient;

pub fn main() -> anyhow::Result<()> {
    let runtime = tokio::runtime::Runtime::new()?;
    let mut client = runtime.block_on(AzookeyServiceClient::connect("http://[::1]:50051"))?;
    println!("Connected to server: {:?}", client);

    let request = tonic::Request::new(protos::proto::AppendTextRequest {
        text_to_append: "kisiniaisatusita".to_string(),
    });

    let response = runtime.block_on(client.append_text(request))?;
    println!(
        "Response from server: {:?}",
        response.into_inner().composing_text
    );

    let request = tonic::Request::new(protos::proto::ClearTextRequest {});
    let response = runtime.block_on(client.clear_text(request))?;
    println!("Response from server: {:?}", response);

    Ok(())
}
