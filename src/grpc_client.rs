use tonic::Request;
use services::{payment_service_client::PaymentServiceClient, PaymentRequest,
    transaction_service_client::TransactionServiceClient,
    TransactionRequest,
    chat_service_client::ChatServiceClient,
    ChatMessage};
use tokio_stream::StreamExt;
use std::io;


pub mod services {
    tonic::include_proto!("services");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = PaymentServiceClient::connect("http://[::1]:50051").await?;

    let request = Request::new(PaymentRequest {
        user_id: "user_123".to_string(),
        amount: 100.0,
    });

    let response = client.process_payment(request).await?;
    println!("RESPONSE={:?}", response.into_inner());

    let mut transaction_client = TransactionServiceClient::connect("http://[::1]:50051").await?;
let request = Request::new(TransactionRequest {
    user_id: "user_123".to_string(),
});

let mut stream = transaction_client
    .get_transaction_history(request)
    .await?
    .into_inner();

while let Some(transaction) = stream.message().await? {
    println!("Transaction: {:?}", transaction);
}

let mut chat_client = ChatServiceClient::connect("http://[::1]:50051").await?;
let (tx, rx) = mpsc::channel(10);

tokio::spawn(async move {
    let stdin = io::stdin();
    let mut reader = io::BufReader::new(stdin).lines();
    
    while let Ok(Some(line)) = reader.next_line().await {
        let message = ChatMessage {
            user_id: "user_123".to_string(),
            message: line,
        };
        tx.send(message).await.unwrap_or_else(|_| {});
    }
});

let request = Request::new(ReceiverStream::new(rx));
let mut response_stream = chat_client.chat(request).await?.into_inner();

while let Some(response) = response_stream.message().await? {
    println!("Server says: {:?}", response);
}
    Ok(())
}