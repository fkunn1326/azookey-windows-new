use protos::proto::azookey_service_client::AzookeyServiceClient;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct AzookeyService {
    client: AzookeyServiceClient<tonic::transport::channel::Channel>,
    runtime: Arc<tokio::runtime::Runtime>,
}

impl Default for AzookeyService {
    fn default() -> Self {
        let runtime = tokio::runtime::Runtime::new()
            .map_err(|e| {
                log::error!("Failed to create runtime: {:#}", e);
            })
            .unwrap();
        let client = runtime
            .block_on(AzookeyServiceClient::connect("http://[::1]:50051"))
            .map_err(|e| {
                log::error!("Failed to connect to server: {:#}", e);
            })
            .unwrap();
        log::debug!("Connected to server: {:?}", client);

        Self {
            client,
            runtime: Arc::new(runtime),
        }
    }
}

impl AzookeyService {
    pub fn append_text(&mut self, text: String) -> anyhow::Result<String> {
        let request = tonic::Request::new(protos::proto::AppendTextRequest {
            text_to_append: text,
        });

        let response = self
            .runtime
            .clone()
            .block_on(self.client.append_text(request))?;
        let composing_text = response.into_inner().composing_text;
        log::debug!("Response from server: {:?}", composing_text);

        if let Some(composing_text) = composing_text {
            Ok(composing_text.suggestions[0].clone())
        } else {
            Err(anyhow::anyhow!("composing_text is None"))
        }
    }

    pub fn remove_text(&mut self) -> anyhow::Result<String> {
        let request = tonic::Request::new(protos::proto::RemoveTextRequest {});
        let response = self
            .runtime
            .clone()
            .block_on(self.client.remove_text(request))?;
        let composing_text = response.into_inner().composing_text;
        log::debug!("Response from server: {:?}", composing_text);

        if let Some(composing_text) = composing_text {
            Ok(composing_text.spell)
        } else {
            Err(anyhow::anyhow!("composing_text is None"))
        }
    }

    pub fn clear_text(&mut self) -> anyhow::Result<()> {
        let request = tonic::Request::new(protos::proto::ClearTextRequest {});
        let response = self
            .runtime
            .clone()
            .block_on(self.client.clear_text(request))?;
        log::debug!("Response from server: {:?}", response);

        Ok(())
    }
}

// pub fn get_azookey() -> anyhow::Result<()> {
//     let runtime = tokio::runtime::Runtime::new()?;
//     let mut client = runtime.block_on(AzookeyServiceClient::connect("http://[::1]:50051"))?;
//     log::debug!("Connected to server: {:?}", client);
//
//     let request = tonic::Request::new(protos::proto::AppendTextRequest {
//         text_to_append: "kisiniaisatusita".to_string(),
//     });
//
//     let response = runtime.block_on(client.append_text(request))?;
//     log::debug!(
//         "Response from server: {:?}",
//         response.into_inner().composing_text
//     );
//
//     let request = tonic::Request::new(protos::proto::ClearTextRequest {});
//     let response = runtime.block_on(client.clear_text(request))?;
//     log::debug!("Response from server: {:?}", response);
//
//     Ok(())
// }
