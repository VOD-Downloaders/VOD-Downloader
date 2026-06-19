use url::Url;

use super::HeaderMap;
use super::RequestError;
use super::RequesterSpecification;

/////////////////////////////////////////////////////
// NativeRequester
/////////////////////////////////////////////////////
pub struct NativeRequester {
    pub specification: RequesterSpecification,
    pub client: reqwest::Client,
}

impl NativeRequester {
    pub fn new(specification: RequesterSpecification) -> Result<Self, RequestError> {
        const MAX_REDIRECTS: usize = 10;

        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::limited(MAX_REDIRECTS))
            .timeout(std::time::Duration::from_secs(specification.max_timeout))
            .connect_timeout(std::time::Duration::from_secs(specification.connect_timeout))
            .user_agent(specification.user_agent.as_str())
            .referer(false)
            .build()
            .map_err(|error| RequestError::FailedToCreate(error.to_string()))?;

        Ok(Self {
            specification: specification,
            client: client,
        })
    }

    pub async fn get_bytes(&self, url: &Url, headers: HeaderMap) -> Result<Vec<u8>, RequestError> {
        let response = self.client.get(url.as_str()).headers(headers.0).send().await.map_err(|error| {
            trace!("Request failed to send due to error: {}", error);
            RequestError::RequestFailedToSend(error.to_string())
        })?;

        if !response.status().is_success() {
            let status = response.status();
            let error = response.error_for_status().unwrap_err();
            trace!("Request failed, status: {}, error: {}", status, error);
            return Err(RequestError::RequestFailed(format!("Status: {}, Error: {}", status, error,)));
        }

        match response.bytes().await {
            Ok(bytes) => Ok(bytes.into()),
            Err(error) => Err(RequestError::FailedToReadBytes(error.to_string())),
        }
    }

    pub async fn get_string(&self, url: &Url, headers: HeaderMap) -> Result<String, RequestError> {
        let bytes = self.get_bytes(url, headers).await?;
        String::from_utf8(bytes).map_err(|_error| RequestError::FailedToConvertBytesToString)
    }

    pub fn get_specification(&self) -> &RequesterSpecification {
        return &self.specification;
    }
}
