use tonic::{transport::Server, Request, Response, Status};
use services::{
    payment_service_server::{PaymentService, PaymentServiceServer}, PaymentRequest, PaymentResponse,
    transaction_service_server::{TransactionService, TransactionServiceServer},
    TransactionRequest, TransactionResponse,
    chat_service_server::{ChatService, ChatServiceServer},
    ChatMessage
};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

pub mod services {
    tonic::include_proto!("services");
}

#[derive(Debug,Default)]
struct MyPaymentService{}
struct MyTransactionService{}
struct MyChatService{}

#[tonic::async_trait]
impl PaymentService for MyPaymentService {
    async fn process_payment(
        &self,
        request: Request<PaymentRequest>,
    ) -> Result<Response<PaymentResponse>, Status> {
        println!("Received payment request: {:?}", request);


        Ok(Response::new(PaymentResponse {
            success: true
        }))
    }
}

#[tonic::async_trait]
impl TransactionService for MyTransactionService {
    async fn get_transaction_history(
        &self,
        request: Request<TransactionRequest>,
    ) -> Result<Response<ReceiverStream<TransactionResponse>>, Status> {
        println!("Received transaction request: {:?}", request);
        
        let (tx, rx) = mpsc::channel(4);
        let user_id = request.into_inner().user_id;

        tokio::spawn(async move {
            for i in 0..30 {
                let transaction = TransactionResponse {
                    transaction_id: format!("txn_{}_{}", user_id, i),
                    status: "COMPLETED".to_string(),
                    amount: 100.0 * (i as f64),
                    timestamp: chrono::Local::now().to_rfc3339(),
                };
                
                tx.send(transaction).await.unwrap_or_else(|_| {});
                if i % 10 == 0 {
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                }
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }
}

#[tonic::async_trait]
impl ChatService for MyChatService {
    async fn chat(
        &self,
        request: Request<tonic::Streaming<ChatMessage>>,
    ) -> Result<Response<ReceiverStream<ChatMessage>>, Status> {
        let mut stream = request.into_inner();
        let (tx, rx) = mpsc::channel(10);

        tokio::spawn(async move {
            while let Ok(Some(message)) = stream.message().await {
                let reply = ChatMessage {
                    user_id: "server".to_string(),
                    message: format!("Echo: {}", message.message),
                };
                tx.send(reply).await.unwrap_or_else(|_| {});
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let payment_service = MyPaymentService::default();

    Server::builder()
        .add_service(PaymentServiceServer::new(payment_service))
        .add_service(TransactionServiceServer::new(MyTransactionService::default()))
        .add_service(ChatServiceServer::new(MyChatService::default()))
        .serve(addr)
        .await?;

    Ok(())

    
}