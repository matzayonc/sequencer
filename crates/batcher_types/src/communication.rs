use std::sync::Arc;

use async_trait::async_trait;
use mockall::predicate::*;
use mockall::*;
use papyrus_proc_macros::handle_response_variants;
use serde::{Deserialize, Serialize};
use starknet_mempool_infra::component_client::{
    ClientError,
    LocalComponentClient,
    RemoteComponentClient,
};
use starknet_mempool_infra::component_definitions::ComponentRequestAndResponseSender;
use thiserror::Error;

use crate::batcher_types::{
    BatcherResult, BuildProposalInput, DecisionReachedInput, GetStreamContentInput, StreamContent
};
use crate::errors::BatcherError;

pub type LocalBatcherClientImpl = LocalComponentClient<BatcherRequest, BatcherResponse>;
pub type RemoteBatcherClientImpl = RemoteComponentClient<BatcherRequest, BatcherResponse>;
pub type BatcherClientResult<T> = Result<T, BatcherClientError>;
pub type BatcherRequestAndResponseSender =
    ComponentRequestAndResponseSender<BatcherRequest, BatcherResponse>;
pub type SharedBatcherClient = Arc<dyn BatcherClient>;

/// Serves as the batcher's shared interface. Requires `Send + Sync` to allow transferring and
/// sharing resources (inputs, futures) across threads.
#[automock]
#[async_trait]
pub trait BatcherClient: Send + Sync {
    async fn build_proposal(&self, input: BuildProposalInput) -> BatcherClientResult<()>;
    async fn get_stream_content(&self, input: GetStreamContentInput) -> BatcherClientResult<StreamContent>;
    async fn decision_reached(&self, input: DecisionReachedInput) -> BatcherClientResult<()>;
}

#[derive(Debug, Serialize, Deserialize)]
pub enum BatcherRequest {
    BuildProposal(BuildProposalInput),
    GetStreamContent(GetStreamContentInput),
    DecisionReached(DecisionReachedInput),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum BatcherResponse {
    BuildProposal(BatcherResult<()>),
    GetStreamContent(BatcherResult<StreamContent>),
    DecisionReached(BatcherResult<()>),
}

#[derive(Clone, Debug, Error)]
pub enum BatcherClientError {
    #[error(transparent)]
    ClientError(#[from] ClientError),
    #[error(transparent)]
    BatcherError(#[from] BatcherError),
}

#[async_trait]
impl BatcherClient for LocalBatcherClientImpl {
    async fn build_proposal(&self, input: BuildProposalInput) -> BatcherClientResult<()> {
        let request = BatcherRequest::BuildProposal(input);
        let response = self.send(request).await;
        handle_response_variants!(BatcherResponse, BuildProposal, BatcherClientError, BatcherError)
    }

    async fn get_stream_content(&self, input: GetStreamContentInput) -> BatcherClientResult<StreamContent> {
        let request = BatcherRequest::GetStreamContent(input);
        let response = self.send(request).await;
        handle_response_variants!(BatcherResponse, GetStreamContent, BatcherClientError, BatcherError)
    }

    async fn decision_reached(&self, input: DecisionReachedInput) -> BatcherClientResult<()> {
        let request = BatcherRequest::DecisionReached(input);
        let response = self.send(request).await;
        handle_response_variants!(BatcherResponse, DecisionReached, BatcherClientError, BatcherError)
    }
}

#[async_trait]
impl BatcherClient for RemoteBatcherClientImpl {
    async fn build_proposal(&self, input: BuildProposalInput) -> BatcherClientResult<()> {
        let request = BatcherRequest::BuildProposal(input);
        let response = self.send(request).await?;
        handle_response_variants!(BatcherResponse, BuildProposal, BatcherClientError, BatcherError)
    }

    async fn get_stream_content(&self, input: GetStreamContentInput) -> BatcherClientResult<StreamContent> {
        let request = BatcherRequest::GetStreamContent(input);
        let response = self.send(request).await?;
        handle_response_variants!(BatcherResponse, GetStreamContent, BatcherClientError, BatcherError)
    }

    async fn decision_reached(&self, input: DecisionReachedInput) -> BatcherClientResult<()> {
        let request = BatcherRequest::DecisionReached(input);
        let response = self.send(request).await?;
        handle_response_variants!(BatcherResponse, DecisionReached, BatcherClientError, BatcherError)
    }
}
