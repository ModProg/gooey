use std::path::Path;

use bonsaidb::{
    core::{connection::StorageConnection, custom_api::Infallible, kv::Kv},
    server::{
        Backend, BackendError, Configuration, ConnectedClient, CustomApiDispatcher, CustomServer,
        DefaultPermissions,
    },
};
use bonsaidb_counter_shared::{
    ExampleApi, GetCounterHandler, IncrementCounterHandler, Request, RequestDispatcher, Response,
    DATABASE_NAME,
};

/// The server's main entrypoint.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Open a `BonsaiDb` server at the given path, allowing all actions to be
    // done over the network connections.
    let server = CustomServer::<Example>::open(
        Path::new("counter-example.bonsaidb"),
        Configuration {
            default_permissions: DefaultPermissions::AllowAll,
            ..Configuration::default()
        },
    )
    .await?;
    server.register_schema::<()>().await?;
    // Create the database if it doesn't exist.
    server.create_database::<()>(DATABASE_NAME, true).await?;
    // Start listening for websockets. This does not return until the server
    // shuts down. If you want to listen for multiple types of traffic, you will
    // need to spawn the tasks.
    server
        .listen_for_websockets_on("127.0.0.1:8081", false)
        .await?;

    Ok(())
}

/// The example database `Backend`.
#[derive(Debug)]
enum Example {}

impl Backend for Example {
    type ClientData = ();
    type CustomApi = ExampleApi;
    type CustomApiDispatcher = ApiDispatcher;
}

impl CustomApiDispatcher<Example> for ApiDispatcher {
    fn new(server: &CustomServer<Example>, _client: &ConnectedClient<Example>) -> Self {
        ApiDispatcher {
            server: server.clone(),
        }
    }
}

/// The dispatcher for API requests.
#[derive(Debug, actionable::Dispatcher)]
#[dispatcher(input = Request)]
struct ApiDispatcher {
    server: CustomServer<Example>,
}

impl RequestDispatcher for ApiDispatcher {
    type Error = BackendError<Infallible>;
    type Output = Response;
}

#[actionable::async_trait]
impl GetCounterHandler for ApiDispatcher {
    /// Returns the current counter value.
    async fn handle(
        &self,
        _permissions: &actionable::Permissions,
    ) -> Result<Response, BackendError<Infallible>> {
        println!("Returning current counter value.");
        let db = self.server.database::<()>(DATABASE_NAME).await.unwrap();

        let value = db
            .get_key("count")
            .into_u64()
            .await
            .unwrap()
            .unwrap_or_default();
        Ok(Response::CounterValue(value))
    }
}

#[actionable::async_trait]
impl IncrementCounterHandler for ApiDispatcher {
    /// Increments the counter, and publishes a message with the new value.
    async fn handle(
        &self,
        _permissions: &actionable::Permissions,
    ) -> Result<Response, BackendError<Infallible>> {
        let db = self.server.database::<()>(DATABASE_NAME).await?;

        let new_value = db.increment_key_by("count", 1_u64).await?;
        self.server
            .broadcast(Ok(Response::CounterValue(new_value)))
            .await;

        Ok(Response::CounterValue(new_value))
    }
}
