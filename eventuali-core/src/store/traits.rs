use crate::{Event, AggregateId, AggregateVersion, Result};
use crate::streaming::EventStreamer;
use async_trait::async_trait;
use std::sync::Arc;

#[async_trait]
pub trait EventStore {
    async fn save_events(&self, events: Vec<Event>) -> Result<()>;
    
    async fn load_events(
        &self,
        aggregate_id: &AggregateId,
        from_version: Option<AggregateVersion>,
    ) -> Result<Vec<Event>>;
    
    async fn load_events_by_type(
        &self,
        aggregate_type: &str,
        from_version: Option<AggregateVersion>,
    ) -> Result<Vec<Event>>;
    
    async fn get_aggregate_version(&self, aggregate_id: &AggregateId) -> Result<Option<AggregateVersion>>;
    
    /// Set the event streamer for publishing events
    fn set_event_streamer(&mut self, streamer: Arc<dyn EventStreamer + Send + Sync>);
}

#[async_trait]
pub trait EventStoreBackend {
    async fn initialize(&mut self) -> Result<()>;
    
    async fn save_events(&self, events: Vec<Event>) -> Result<()>;
    
    async fn load_events(
        &self,
        aggregate_id: &AggregateId,
        from_version: Option<AggregateVersion>,
    ) -> Result<Vec<Event>>;
    
    async fn load_events_by_type(
        &self,
        aggregate_type: &str,
        from_version: Option<AggregateVersion>,
    ) -> Result<Vec<Event>>;
    
    async fn get_aggregate_version(&self, aggregate_id: &AggregateId) -> Result<Option<AggregateVersion>>;
}

pub trait EventSerializer {
    fn serialize_event_data(&self, event: &Event) -> Result<Vec<u8>>;
    fn deserialize_event_data(&self, data: &[u8], event_type: &str) -> Result<Event>;
}