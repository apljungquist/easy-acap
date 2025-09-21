use std::{ffi::CString, time::SystemTime};

use crate::{utils::cstring_from_str_lossy, Error};

/// Provides the associate topic, which all events must have.
pub trait StructuredEvent {
    fn topic() -> KeyValueSetBuilder;
}

#[non_exhaustive]
#[derive(Debug)]
enum Tag {
    Data,
    Source,
    UserDefined(CString),
}

/// A writable event object given to a [`crate::Publisher`].
#[derive(Debug, Default)]
pub struct EventBuilder {
    key_value_set: KeyValueSetBuilder,
    timestamp: Option<SystemTime>,
}

impl EventBuilder {
    pub fn insert(self, key_value: KeyValuePair) -> Self {
        Self {
            key_value_set: self.key_value_set.insert(key_value),
            timestamp: self.timestamp,
        }
    }

    pub fn timestamp(mut self, timestamp: SystemTime) -> Self {
        self.timestamp = Some(timestamp);
        self
    }

    pub fn build(self) -> Result<Event, Error> {
        Ok(Event(axevent::flex::Event::new2(
            self.key_value_set.build()?,
            self.timestamp.map(axevent::ergo::date_time),
        )))
    }
}

/// A readable event object received from a [`crate::Subscriber`].
pub struct Event(axevent::flex::Event);

impl Event {
    pub(crate) fn into_inner(self) -> axevent::flex::Event {
        self.0
    }
}

impl Event {
    pub fn builder() -> EventBuilder {
        EventBuilder::default()
    }

    pub(crate) fn from_event(event: axevent::flex::Event) -> Self {
        Self(event)
    }

    pub fn timestamp(&self) -> SystemTime {
        axevent::ergo::system_time(self.0.time_stamp2())
    }

    pub fn try_get_boolean(
        &self,
        key: &str,
        namespace: Option<&str>,
    ) -> Result<Option<bool>, Error> {
        let key = cstring_from_str_lossy(key);
        let namespace = namespace.map(cstring_from_str_lossy);
        self.0
            .key_value_set()
            .get_boolean(key.as_ref(), namespace.as_deref())
    }

    pub fn try_get_double(&self, key: &str, namespace: Option<&str>) -> Result<Option<f64>, Error> {
        let key = cstring_from_str_lossy(key);
        let namespace = namespace.map(cstring_from_str_lossy);
        self.0
            .key_value_set()
            .get_double(key.as_ref(), namespace.as_deref())
    }

    pub fn try_get_string(
        &self,
        key: &str,
        namespace: Option<&str>,
    ) -> Result<Option<String>, Error> {
        let key = cstring_from_str_lossy(key);
        let namespace = namespace.map(cstring_from_str_lossy);
        self.0
            .key_value_set()
            .get_string(key.as_ref(), namespace.as_deref())
            .map(|s| s.map(|s| s.as_c_str().to_string_lossy().to_string()))
    }
}

/// A value that can be added to a [`KeyValuePair`].
#[non_exhaustive]
#[derive(Debug)]
pub enum Value {
    String(CString),
    Double(f64),
    Integer(i32),
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Self::String(cstring_from_str_lossy(s))
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Self::Double(value)
    }
}

impl From<u8> for Value {
    fn from(u: u8) -> Self {
        Self::Integer(u.into())
    }
}

/// A key-value pair builder that can be used to build a [`KeyValueSetBuilder`].
#[derive(Debug)]
pub struct KeyValuePair {
    key: CString,
    value: Option<Value>,
    namespace: Option<CString>,
    tag: Option<Tag>,
    key_nice_name: Option<CString>,
    value_nice_name: Option<CString>,
}

impl KeyValuePair {
    /// Create a new key-value-pair.
    pub fn new(key: &str) -> Self {
        Self {
            key: cstring_from_str_lossy(key),
            value: None,
            namespace: None,
            tag: None,
            key_nice_name: None,
            value_nice_name: None,
        }
    }

    /// Set the namespace on this key-value-pair.
    pub fn namespace(mut self, namespace: &str) -> Self {
        self.namespace = Some(cstring_from_str_lossy(namespace));
        self
    }

    /// Set the value on this key-value-pair.
    pub fn value<T>(mut self, value: T) -> Self
    where
        T: Into<Value>,
    {
        self.value = Some(value.into());
        self
    }

    /// Set the tag on this key-value-pair.
    ///
    /// Note that only events with **exactly one** data key can be used to trigger actions.
    pub fn data(mut self) -> Self {
        self.tag = Some(Tag::Data);
        self
    }

    /// Set the tag on this key-value-pair.
    ///
    /// Note that only events with **at most one** source key can be used to trigger actions.
    pub fn source(mut self) -> Self {
        self.tag = Some(Tag::Source);
        self
    }

    /// Set the tag on this key-value-pair.
    pub fn user_defined(mut self, tag: &str) -> Self {
        self.tag = Some(Tag::UserDefined(cstring_from_str_lossy(tag)));
        self
    }

    fn apply(self, kvs: &mut axevent::flex::KeyValueSet) -> Result<(), axevent::flex::Error> {
        let Self {
            key,
            value,
            namespace,
            tag,
            key_nice_name,
            value_nice_name,
        } = self;
        match value {
            // FIXME: Propagate type of None? (added a test to run when I have camera)
            // What happens if we try to read a None value from a KVS?
            None => kvs.add_key_value::<bool>(key.as_c_str(), namespace.as_deref(), None),
            Some(Value::String(s)) => {
                kvs.add_key_value(key.as_c_str(), namespace.as_deref(), Some(s.as_c_str()))
            }
            Some(Value::Double(d)) => {
                kvs.add_key_value(key.as_c_str(), namespace.as_deref(), Some(d))
            }
            Some(Value::Integer(i)) => {
                kvs.add_key_value(key.as_c_str(), namespace.as_deref(), Some(i))
            }
        }?;
        match tag {
            None => {}
            Some(Tag::Data) => {
                kvs.mark_as_data(key.as_c_str(), namespace.as_deref())?;
            }
            Some(Tag::Source) => {
                kvs.mark_as_source(key.as_c_str(), namespace.as_deref())?;
            }
            Some(Tag::UserDefined(s)) => {
                kvs.mark_as_user_defined(key.as_c_str(), namespace.as_deref(), s.as_c_str())?;
            }
        };
        kvs.add_nice_names(
            key.as_c_str(),
            namespace.as_deref(),
            key_nice_name.as_deref(),
            value_nice_name.as_deref(),
        )?;
        Ok(())
    }
}

/// A key-value set builder that can be used to publish and subscribe to events.
#[derive(Debug, Default)]
pub struct KeyValueSetBuilder {
    inner: Vec<KeyValuePair>,
}

impl KeyValueSetBuilder {
    /// Add a key-value pair to the set.
    ///
    /// Note that each key should be unique and duplicate keys will be discarded.
    /// The namespace is disregarded when determining the uniqueness of a key,
    /// but it is required when determining if a key is a match when publishing and subscribing.
    pub fn insert(mut self, key_value: KeyValuePair) -> Self {
        self.inner.push(key_value);
        self
    }

    pub(crate) fn is_stateless(&self) -> bool {
        self.inner.iter().any(|kv| kv.value.is_none())
    }

    pub(crate) fn union(mut self, mut other: KeyValueSetBuilder) -> Self {
        for kv in other.inner.drain(..) {
            self.inner.push(kv);
        }
        self
    }

    pub fn build(mut self) -> Result<axevent::flex::KeyValueSet, axevent::flex::Error> {
        let mut kvs = axevent::flex::KeyValueSet::new();
        for kv in self.inner.drain(..) {
            kv.apply(&mut kvs)?;
        }
        Ok(kvs)
    }
}
