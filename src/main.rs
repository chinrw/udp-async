use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::UdpSocket;
use tokio::runtime::Handle;
use tokio::sync::mpsc;
use tokio::task;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server_address = "127.0.0.1:9090";
    let send_numbers = 1000000;
    let num_tasks = 4; // Number of tasks
    let sockets_per_task = 20; // Number of sockets per task

    let (tx, mut rx) = mpsc::channel::<()>(num_tasks);

    let start = Instant::now();
    let mut tasks = vec![];
    for _ in 0..num_tasks {
        let tx = tx.clone();
        let server_address = server_address.to_string();

        let task = task::spawn(async move {
            let mut sockets = Vec::new();
            for _ in 0..sockets_per_task {
                let socket = UdpSocket::bind("0.0.0.0:0").await.unwrap();
                sockets.push(Arc::new(socket));
            }

            for i in 0..send_numbers / num_tasks / sockets_per_task {
                for socket in &sockets {
                    let message = format!("Hello, World! {}", i);
                    let socket = Arc::clone(&socket);
                    socket
                        .send_to(message.as_bytes(), &server_address)
                        .await
                        .unwrap();
                    task::spawn(async move {
                        let mut buffer = [0; 1024];
                        let (len, _) = socket.recv_from(&mut buffer).await.unwrap();
                        // println!("Received: {}", String::from_utf8_lossy(&buffer[..len]));
                    })
                    .await
                    .unwrap();
                }
            }
            // drop(sockets); // Explicitly drop sockets if needed
            // let _ = tx.send(()).await;
        });
        tasks.push(task);
    }
    for task in tasks {
        let metrics = Handle::current().metrics();

        let n = metrics.active_tasks_count();
        println!("Runtime has {} active tasks", n);
        task.await.unwrap();
    }

    // for _ in 0..num_tasks {
    //     rx.recv().await;
    // }
    let duration = start.elapsed();

    println!("Time elapsed in expensive_function() is: {:?}", duration);

    Ok(())
}
