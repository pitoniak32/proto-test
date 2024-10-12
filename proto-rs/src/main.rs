pub mod proto {
    tonic::include_proto!("calculator");

    pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("calculator_descriptor");
}

use proto::calculator_server::{Calculator, CalculatorServer};
use tonic::transport::Server;

pub mod otel;

#[derive(Debug, Default)]
struct CalculatorService {}

#[tonic::async_trait]
impl Calculator for CalculatorService {
    #[tracing::instrument(skip_all)]
    async fn add(
        &self,
        request: tonic::Request<proto::CalculationRequest>,
    ) -> Result<tonic::Response<proto::CalculationResponse>, tonic::Status> {
        tracing::info!("Got request: {request:?}");

        let input = request.get_ref();

        let response = proto::CalculationResponse {
            result: input.a + input.b,
        };

        Ok(tonic::Response::new(response))
    }

    #[tracing::instrument(skip_all)]
    async fn divide(
        &self,
        request: tonic::Request<proto::CalculationRequest>,
    ) -> Result<tonic::Response<proto::CalculationResponse>, tonic::Status> {
        tracing::info!("Got request: {request:?}");

        let input = request.get_ref();

        if input.b == 0 {
          tracing::warn!("Attemped to divide by zero");
          return Err(tonic::Status::invalid_argument("cannot divide by zero"))
        }

        let response = proto::CalculationResponse {
            result: input.a / input.b,
        };

        Ok(tonic::Response::new(response))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = otel::setup_otel("grpc://localhost:4317", "test-grpc-server");
    
    let addr = "[::1]:50051".parse()?;

    let calc = CalculatorService::default();

    tracing::info!("setting up reflection");
    let service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(proto::FILE_DESCRIPTOR_SET)
        .build_v1()?;

    tracing::info!("listening on: {addr:?}");
    Server::builder()
        .add_service(service)
        .add_service(CalculatorServer::new(calc))
        .serve(addr)
        .await?;

    Ok(())
}
