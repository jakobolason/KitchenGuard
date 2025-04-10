use std::collections::HashMap;
use std::time::Duration;
use futures::future::{ready, Ready};
use futures::Future;

use mongodb::Client;

pub struct GuardSessionStorage;

impl SessionStore for GuardSessionStorage {
    fn load(
        &self,
        _session_key: &SessionKey,
    ) -> impl Future<Output = Result<Option<HashMap<String, String>>, LoadError>> {
        // query db for this sessionkey if it exists
        let uri = std::env::var("MONGODB_URI").unwrap_or_else(|_| "mongodb://localhost:27017".into());

        // let all workers use the client/Mongodb connection
        let client = Client::with_uri_str(uri).await.expect("failed to connect");

        let collection = client.database(SessionStorage).collection(Sessions);
        let result = collection.insert_one(_session_key).await;
    }

    fn save(
        &self,
        _session_state: HashMap<String, String>,
        _ttl: &Duration,
    ) -> impl Future<Output = Result<SessionKey, SaveError>> {
        ready(Err(SaveError::new("Not implemented"))) // Example implementation
    }

    fn update(
        &self,
        _session_key: SessionKey,
        _session_state: HashMap<String, String>,
        _ttl: &Duration,
    ) -> impl Future<Output = Result<SessionKey, UpdateError>> {
        ready(Err(UpdateError::new("Not implemented"))) // Example implementation
    }

    fn update_ttl(
        &self,
        _session_key: &SessionKey,
        _ttl: &Duration,
    ) -> impl Future<Output = Result<(), Error>> {
        ready(Err(Error::new("Not implemented"))) // Example implementation
    }

    fn delete(
        &self,
        _session_key: &SessionKey,
    ) -> impl Future<Output = Result<(), Error>> {
        ready(Err(Error::new("Not implemented"))) // Example implementation
    }
}