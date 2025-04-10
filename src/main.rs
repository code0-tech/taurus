use futures_lite::stream::StreamExt;
use lapin::{
    options::{BasicAckOptions, BasicConsumeOptions, QueueDeclareOptions},
    types::FieldTable,
    Channel, Connection,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Serialize, Deserialize)]
enum MessageType {
    ExecuteFlow,
    TestExecuteFlow,
}

#[derive(Serialize, Deserialize)]
struct Sender {
    name: String,
    protocol: String,
    version: String,
}

#[derive(Serialize, Deserialize)]
struct Message {
    message_type: MessageType,
    sender: Sender,
    timestamp: i64,
    telegram_id: String,
    body: String,
}

async fn build_connection(rabbitmq_url: &str) -> Connection {
    match Connection::connect(rabbitmq_url, lapin::ConnectionProperties::default()).await {
        Ok(env) => env,
        Err(error) => panic!(
            "Cannot connect to FlowQueue (RabbitMQ) instance! Reason: {:?}",
            error
        ),
    }
}

// Thread-safe wrapper for RabbitMQ channel
struct RabbitmqClient {
    channel: Arc<Mutex<Channel>>,
}

impl RabbitmqClient {
    // Create a new RabbitMQ client with channel
    async fn new(rabbitmq_url: &str) -> Self {
        let connection = build_connection(rabbitmq_url).await;
        let channel = connection.create_channel().await.unwrap();

        // Declare the queue once during initialization
        channel
            .queue_declare(
                "send_queue",
                QueueDeclareOptions::default(),
                FieldTable::default(),
            )
            .await
            .unwrap();

        channel
            .queue_declare(
                "recieve_queue",
                QueueDeclareOptions::default(),
                FieldTable::default(),
            )
            .await
            .unwrap();

        RabbitmqClient {
            channel: Arc::new(Mutex::new(channel)),
        }
    }

    // Send message to the queue
    async fn send_message(&self, message_json: String, queue_name: &str) {
        let channel = self.channel.lock().await;

        channel
            .basic_publish(
                "",         // exchange
                queue_name, // routing key (queue name)
                lapin::options::BasicPublishOptions::default(),
                message_json.as_bytes(),
                lapin::BasicProperties::default(),
            )
            .await
            .expect("TEST");
    }

    // Receive messages from a queue
    async fn receive_messages(&self, queue_name: &str) -> Result<(), lapin::Error> {
        let mut consumer = {
            let channel = self.channel.lock().await;

            let consumer_res = channel
                .basic_consume(
                    queue_name,
                    "consumer",
                    BasicConsumeOptions::default(),
                    FieldTable::default(),
                )
                .await;

            match consumer_res {
                Ok(consumer) => consumer,
                Err(err) => panic!("{}", err),
            }
        };

        println!("Starting to consume from {}", queue_name);

        while let Some(delivery) = consumer.next().await {
            let delivery = match delivery {
                Ok(del) => del,
                Err(err) => {
                    println!("Error receiving message: {}", err);
                    return Err(err);
                }
            };

            let data = &delivery.data;
            let message_str = match std::str::from_utf8(&data) {
                Ok(str) => {
                    println!("Received message: {}", str);
                    str
                }
                Err(err) => {
                    println!("Error decoding message: {}", err);
                    return Ok(());
                }
            };
            // Parse the message
            let message = match serde_json::from_str::<Message>(message_str) {
                Ok(mess) => {
                    println!("Parsed message with telegram_id: {}", mess.telegram_id);
                    mess
                }
                Err(err) => {
                    println!("Error parsing message: {}", err);
                    return Ok(());
                }
            };

            // Process the message here
            let hello_world = Message {
                telegram_id: message.telegram_id,
                message_type: message.message_type,
                timestamp: message.timestamp,
                sender: message.sender,
                body: "{ \"text\": \"Hello, World!\" }".to_string(),
            };

            let hello_world_json = serde_json::to_string(&hello_world).unwrap();

            println!("{}", hello_world_json);

            {
                self.send_message(hello_world_json, "recieve_queue").await;
            }

            // Acknowledge the message
            delivery
                .ack(BasicAckOptions::default())
                .await
                .expect("Failed to acknowledge message");
        }

        Ok(())
    }
}

#[tokio::main]
async fn main() {
    let rabbitmq_client = Arc::new(RabbitmqClient::new("amqp://localhost:5672").await);

    // Receive messages from the send_queue
    if let Err(e) = rabbitmq_client.receive_messages("send_queue").await {
        eprintln!("Failed to receive messages: {}", e);
    }
}
