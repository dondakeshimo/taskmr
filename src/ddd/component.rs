use anyhow::Result;
use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use std::cmp::Eq;
use std::fmt;
use uuid::Uuid;

/// Marker trait represents ValueObject in DDD.
pub trait ValueObject: PartialEq + Eq + Clone + Send + Sync {}

/// Marker trait represents Entity in DDD.
pub trait Entity {
    type Id: ValueObject;

    /// return id.
    /// id must unique in defined context.
    fn id(&self) -> Self::Id;
}

/// Command is the message to mutate a Aggregate.
pub trait Command: Send + Sync {}

/// DomainEvent is the message what is happend.
pub trait DomainEvent: Send + Sync + Serialize {}

/// DomainEventEnvelope is to add metadata to DomainEvent.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DomainEventEnvelope<E: DomainEvent> {
    event: E,
    aggregate_version: i32,
    event_version: i32,
    occurred_on: NaiveDateTime,
}

impl<E: DomainEvent> DomainEventEnvelope<E> {
    /// construct TaskDomainEventEnvelope.
    pub fn new(event: E, aggregate_version: i32, event_version: i32) -> Self {
        Self {
            event,
            aggregate_version,
            event_version,
            occurred_on: Utc::now().naive_utc(),
        }
    }

    /// get event.
    pub fn event(&self) -> &E {
        &self.event
    }

    /// get aggregate_version.
    pub fn aggregate_version(&self) -> i32 {
        self.aggregate_version
    }

    /// get event_version.
    pub fn event_version(&self) -> i32 {
        self.event_version
    }

    /// get occurred_on.
    pub fn occurred_on(&self) -> NaiveDateTime {
        self.occurred_on
    }
}

/// Aggregate ID.
/// This ID is generated at the same time when the task is created.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AggregateID(Uuid);

impl AggregateID {
    /// construct a AggregateID.
    pub fn new() -> Self {
        AggregateID(Uuid::new_v4())
    }
}

impl Default for AggregateID {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for AggregateID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl ValueObject for AggregateID {}

/// AggregateRoot is the Entity which receive Commands and trigger DomainEvents.
pub trait AggregateRoot: Entity {
    type Command: Command;
    type DomainEvent: DomainEvent;

    /// execute Command.
    /// This function is typically called Command Handler.
    fn execute(&mut self, commands: Self::Command) -> Result<()>;

    /// apply DomainEvent.
    /// This function is typically called DomainEvent Handler.
    fn apply(&mut self, event: &Self::DomainEvent);

    /// get events.
    fn events(&self) -> &Vec<DomainEventEnvelope<Self::DomainEvent>>;

    /// clear events.
    fn clear_events(&mut self);

    /// record_event mutate the aggregate, store the event to the aggregate and increment aggregate_version.
    fn record_event(&mut self, event: Self::DomainEvent);
}

/// Repository returns AggregateRoot to a client.
/// Repository should not be invoked on Entity.
pub trait Repository<AR: AggregateRoot> {
    /// load Event Sourced AggregateRoot from EventStore.
    fn load(&self, id: AR::Id) -> Result<AR>;

    /// save Event Sourced AggregateRoot as DomainEvent Stream and increment EA Version.
    /// NOTE: don't forget invoke `clear_events` method of AggregateRoot after save to Event Store.
    fn save(&self, root: &mut AR) -> Result<()>;
}
