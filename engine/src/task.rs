use std::sync::Arc;
use tdn::prelude::{GroupId, SendMessage};
use tokio::{
    select,
    sync::{
        mpsc::{channel, Receiver, Sender, UnboundedSender},
        Mutex,
    },
    time::sleep,
};

use crate::{
    engine::{handle_result, HandlerRoom},
    types::ChainMessage,
    Handler, Task, Tasks,
};

pub enum TaskMessage {
    Close,
}

pub async fn handle_tasks<H: Handler>(
    room_id: GroupId,
    room: Arc<Mutex<HandlerRoom<H>>>,
    send: Sender<SendMessage>,
    chain_send: UnboundedSender<ChainMessage>,
    mut recv: Receiver<TaskMessage>,
    tasks: Tasks<H>,
) {
    let mut senders = vec![];
    for task in tasks {
        let (tx, rx) = channel(1);
        let room1 = room.clone();
        let send1 = send.clone();
        let chain_send1 = chain_send.clone();
        senders.push(tx);
        tokio::spawn(running(room_id, room1, send1, chain_send1, rx, task));
    }

    loop {
        match recv.recv().await {
            Some(message) => match message {
                TaskMessage::Close => {
                    for tx in &senders {
                        let _ = tx.send(TaskMessage::Close).await;
                    }
                }
            },
            None => break,
        }
    }
}

enum FutureMessage {
    Next,
    Out(TaskMessage),
}

async fn running<H: Handler>(
    room_id: GroupId,
    room: Arc<Mutex<HandlerRoom<H>>>,
    send: Sender<SendMessage>,
    chain_send: UnboundedSender<ChainMessage>,
    mut recv: Receiver<TaskMessage>,
    mut task: Box<dyn Task<H = H>>,
) {
    loop {
        let work = select! {
            w = async {
                recv.recv().await.map(FutureMessage::Out)
            } => w,
            w = async {
                sleep(std::time::Duration::from_secs(task.timer())).await;
                Some(FutureMessage::Next)
            } => w,
        };

        match work {
            Some(FutureMessage::Out(message)) => match message {
                TaskMessage::Close => break,
            },
            Some(FutureMessage::Next) => {
                let mut room_lock = room.lock().await;
                if let Ok(mut res) = task.run(&mut room_lock.handler).await {
                    let over = res.replace_over();
                    handle_result(&room_lock.room, res, &send, None).await;
                    if let Some((data, proof)) = over {
                        let _ = chain_send.send(ChainMessage::OverRoom(room_id, data, proof));
                    }
                }
                drop(room_lock);
            }
            None => break,
        }
    }
}
