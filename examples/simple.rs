use truba::{Context, Message, MpscChannel, Receiver};

struct Value(u32);

impl Message for Value {
    type Channel = MpscChannel<Self>;
}

struct MyActor {
    value: u32,
    value_in: Receiver<Value>,
}

impl MyActor {
    fn spawn_event_loop(mut self, ctx: Context) {
        truba::spawn_event_loop!(ctx, {
            Some(msg) = self.value_in.recv() => {
                self.handle_value(msg);
            },
            else => break,
        });
    }
}

impl MyActor {
    fn run(ctx: Context, value: u32) {
        let value_in = ctx.receiver::<Value>();
        let actor = MyActor { value, value_in };

        actor.spawn_event_loop(ctx)
    }
}

impl MyActor {
    fn handle_value(&mut self, Value(value): Value) {
        self.value = value;
        println!("receive value {value}");
    }
}

#[tokio::main]
async fn main() {
    let ctx = Context::default();
    MyActor::run(ctx.clone(), 42);

    ctx.sender::<Value>().send(Value(22)).await.ok();

    ctx.shutdown().await;
}
