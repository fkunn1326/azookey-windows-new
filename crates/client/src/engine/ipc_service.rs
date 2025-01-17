use protos::proto::{
    azookey_service_client::AzookeyServiceClient, window_service_client::WindowServiceClient,
};
use std::sync::Arc;

// connect to kkc server
#[derive(Debug, Clone)]
pub struct IPCService {
    // kkc server client
    azookey_client: AzookeyServiceClient<tonic::transport::channel::Channel>,
    // candidate window server client
    window_client: WindowServiceClient<tonic::transport::channel::Channel>,
    runtime: Arc<tokio::runtime::Runtime>,
}

impl Default for IPCService {
    fn default() -> Self {
        let runtime = tokio::runtime::Runtime::new()
            .map_err(|e| {
                log::error!("Failed to create runtime: {:#}", e);
            })
            .unwrap();
        let azookey_client = runtime
            .block_on(AzookeyServiceClient::connect("http://[::1]:50051"))
            .map_err(|e| {
                log::error!("Failed to connect to server: {:#}", e);
            })
            .unwrap();
        let window_client = runtime
            .block_on(WindowServiceClient::connect("http://[::1]:50052"))
            .map_err(|e| {
                log::error!("Failed to connect to server: {:#}", e);
            })
            .unwrap();
        log::debug!("Connected to server: {:?}", azookey_client);

        Self {
            azookey_client,
            window_client,
            runtime: Arc::new(runtime),
        }
    }
}

// implement methods to interact with kkc server
impl IPCService {
    pub fn append_text(&mut self, text: String) -> anyhow::Result<String> {
        let request = tonic::Request::new(protos::proto::AppendTextRequest {
            text_to_append: text,
        });

        let response = self
            .runtime
            .clone()
            .block_on(self.azookey_client.append_text(request))?;
        let composing_text = response.into_inner().composing_text;

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
            .block_on(self.azookey_client.remove_text(request))?;
        let composing_text = response.into_inner().composing_text;

        if let Some(composing_text) = composing_text {
            Ok(composing_text.spell)
        } else {
            Err(anyhow::anyhow!("composing_text is None"))
        }
    }

    pub fn clear_text(&mut self) -> anyhow::Result<()> {
        let request = tonic::Request::new(protos::proto::ClearTextRequest {});
        let _response = self
            .runtime
            .clone()
            .block_on(self.azookey_client.clear_text(request))?;

        Ok(())
    }
}

// implement methods to interact with candidate window server
impl IPCService {
    pub fn show_window(&mut self) -> anyhow::Result<()> {
        let request = tonic::Request::new(protos::proto::EmptyResponse {});
        self.runtime
            .clone()
            .block_on(self.window_client.show_window(request))?;

        Ok(())
    }

    pub fn hide_window(&mut self) -> anyhow::Result<()> {
        let request = tonic::Request::new(protos::proto::EmptyResponse {});
        self.runtime
            .clone()
            .block_on(self.window_client.hide_window(request))?;

        Ok(())
    }

    pub fn set_window_position(&mut self, x: i32, y: i32) -> anyhow::Result<()> {
        let request = tonic::Request::new(protos::proto::SetPositionRequest {
            position: Some(protos::proto::WindowPosition { x, y }),
        });
        self.runtime
            .clone()
            .block_on(self.window_client.set_window_position(request))?;

        Ok(())
    }
}
