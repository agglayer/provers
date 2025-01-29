// use std::str::FromStr;
//
// use tonic::transport::Endpoint;
// use tracing::error;
//
// use crate::config::ProposerClientConfig;
// use crate::proposer_v1::{
//     proposer_service_client::ProposerServiceClient, GetProofsRequest,
// GetProofsResponse, };
// use crate::{Request, Response};
//
// pub type ProposerGrpcClient =
// ProposerServiceClient<tonic::transport::Channel>;
//
// pub(crate) fn connect_proposer_service(
//     config: &ProposerClientConfig,
// ) -> Result<ProposerGrpcClient, tonic::transport::Error> {
//     let endpoint = format!("{}:{}", config.host, config.port);
//     match Endpoint::from_str(&endpoint) {
//         Ok(endpoint) =>
// Ok(ProposerServiceClient::new(endpoint.connect_lazy())),         Err(e) => {
//             error!(endpoint, "Failure to setup the gRPC API with endpoint:
// {e}");             Err(e)
//         }
//     }
// }
//
// impl From<GetProofsRequest> for Request {
//     fn from(req: GetProofsRequest) -> Self {
//         Request {
//             start_block: req.start_block,
//             end_block: req.end_block,
//         }
//     }
// }
//
// impl From<Request> for GetProofsRequest {
//     fn from(req: Request) -> Self {
//         GetProofsRequest {
//             start_block: req.start_block,
//             end_block: req.end_block,
//         }
//     }
// }
//
// impl From<GetProofsResponse> for Response {
//     fn from(res: GetProofsResponse) -> Self {
//         Response { proofs: res.proof }
//     }
// }
//
// impl From<Response> for GetProofsResponse {
//     fn from(res: Response) -> Self {
//         GetProofsResponse { proof: res.proofs }
//     }
// }
