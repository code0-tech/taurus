use code0_flow::flow_queue::service::{Message, RabbitmqClient};
use std::sync::Arc;

fn handle_message(message: Message) -> Result<Message, lapin::Error> {
    Ok(Message {
        message_id: message.message_id,
        message_type: message.message_type,
        timestamp: message.timestamp,
        sender: message.sender,
        body: "{ \"text\": \"Hihi, World!\" }".to_string(),
    })
}

#[tokio::main]
async fn main() {
    let rabbitmq_client = Arc::new(RabbitmqClient::new("amqp://localhost:5672").await);

    // Receive messages from the send_queue
    if let Err(e) = rabbitmq_client
        .receive_messages("send_queue", handle_message)
        .await
    {
        eprintln!("Failed to receive messages: {}", e);
    }
}
