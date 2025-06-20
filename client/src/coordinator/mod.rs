use std::error::Error;

mod ws;

trait ICoordinator {
    type Message;
    type Response;
    type Event;

    async fn send(&mut self, message: Self::Message) -> Result<Self::Response, Box<dyn Error>>;

    async fn connect(
        &mut self,
        url: String,
    ) -> Result<tokio::sync::mpsc::UnboundedReceiver<Self::Event>, Box<dyn Error>>;
}
