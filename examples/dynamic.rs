use truba::{Context, Message, MpscChannel, TaskId};

struct Value(u32);

impl Message for Value {
    type Channel = MpscChannel<Self>;
}

struct MyActor {
    value: u32,
}

impl MyActor {
    fn run(ctx: Context, actor_id: &'static str, value: u32) -> TaskId {
        let mut value_in = ctx.actor_receiver::<Value>(actor_id);
        let mut actor = MyActor { value };

        println!("run the {actor_id}");
        ctx.clone().spawn(async move {
            truba::event_loop!(ctx, {
                Some(msg) = value_in.recv() => {
                    actor.handle_value(msg);
                    if actor.value == 0 {
                        break;
                    }
                },
            });
            println!("stop the {actor_id}");
        })
    }

    fn handle_value(&mut self, Value(value): Value) {
        self.value = value;
        println!("receive value {value}");
    }
}

#[tokio::main]
async fn main() {
    let ctx = Context::new();

    MyActor::run(ctx.clone(), "actor 1", 1);
    let actor_2_task = MyActor::run(ctx.clone(), "actor 2", 2);

    let sender_1 = ctx.actor_sender::<Value>("actor 1");
    sender_1.send(Value(11)).await.ok();
    sender_1.send(Value(12)).await.ok();

    let sender_2 = ctx.actor_sender::<Value>("actor 2");
    sender_2.send(Value(21)).await.ok();
    sender_2.send(Value(22)).await.ok();

    sender_2.send(Value(0)).await.ok();
    ctx.join(&actor_2_task).await.ok();

    MyActor::run(ctx.clone(), "actor 2", 2);

    let sender_2 = ctx.actor_sender::<Value>("actor 2");
    sender_2.send(Value(23)).await.ok();

    ctx.shutdown().await;
}
