use proto::calculator_client::CalculatorClient;

pub mod proto {
  tonic::include_proto!("calculator");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "http://[::1]:50051";
    let mut client = CalculatorClient::connect(addr).await?;
    let req = proto::CalculationRequest { a: 4, b: 0 };
    let request = tonic::Request::new(req);

    let response = client.divide(request).await?;

    println!("response: {:?}", response.get_ref());

    Ok(())
}