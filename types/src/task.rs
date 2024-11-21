use std::sync::Arc;
use tokio::{
    sync::{mpsc::UnboundedSender, Mutex},
    time::sleep,
};

use crate::{HandleResult, Handler, RoomId, Task, Tasks};

/// Task message type
pub enum TaskMessage<H: Handler> {
    Result(RoomId, HandleResult<H::Param>),
}

/// Handle and listening tasks
pub fn handle_tasks<H: Handler>(
    room_id: RoomId,
    tasks: Tasks<H>,
    handler: Arc<Mutex<H>>,
    sender: UnboundedSender<TaskMessage<H>>,
) {
    for task in tasks {
        tokio::spawn(running(room_id, task, handler.clone(), sender.clone()));
    }
}

/// Loop listening task
async fn running<H: Handler>(
    room_id: RoomId,
    mut task: Box<dyn Task<H = H>>,
    handler: Arc<Mutex<H>>,
    sender: UnboundedSender<TaskMessage<H>>,
) {
    loop {
        sleep(std::time::Duration::from_secs(task.timer())).await;

        let mut handler_lock = handler.lock().await;
        if let Ok(res) = task.run(&mut handler_lock).await {
            let over = res.over;
            let _ = sender.send(TaskMessage::Result(room_id, res));
            if over {
                drop(handler_lock);
                break;
            }
        } else {
            drop(handler_lock);
            break;
        }
        drop(handler_lock);
    }
}
