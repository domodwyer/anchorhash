use std::{
    collections::hash_map::RandomState,
    convert::TryFrom,
    default::Default,
    hash::{BuildHasher, Hash, Hasher},
    iter::FromIterator,
    marker::PhantomData,
};

use hashbrown::HashMap;
use thiserror::Error;

use crate::{anchor::Anchor, ResourceIterator, ResourceMutIterator};

/// Errors returned when operating on an [`AnchorHash`] instance.
#[derive(Debug, Error, PartialEq, Clone, Copy)]
pub enum Error {
    /// A new bucket cannot be added to the AnchorHash instance as it has
    /// reached the configured capacity.
    #[error("configured resource capacity reached")]
    CapacityLimitReached,

    /// The requested resource is not registered with the AnchorHash instance.
    #[error("resource not found")]
    ResourceNotFound,
}

type Result<T, E = Error> = std::result::Result<T, E>;

/// Initialise a new [`AnchorHash`] instance.
///
/// An AnchorHash instance can be pre-populated with some set of resources using
/// [`with_resources`]:
///
/// ```rust
/// // Anything can be used as the resource!
/// //
/// // Here &str instances are used, but SockAddr instances, structs, Strings,
/// // or anything that makes sense for you can be used too.
/// let backend_servers = vec![
///     "cache1.itsallbroken.com",
///     "cache2.itsallbroken.com",
///     "cache3.itsallbroken.com",
/// ];
///
/// let anchor = anchorhash::Builder::default()
///     .with_resources(backend_servers)
///     .build(100);
///
/// // In this example, strings are used as keys but anything that implements
/// // std::hash::Hash can be used.
/// let example_user_email = "dom@itsallbroken.com".to_string();
///
/// // Get a backend for this user
/// let backend = anchor.get_resource(example_user_email).unwrap();
///
/// println!("user mapped to: {}", backend);
/// ```
///
/// By default, a builder constructs an empty AnchorHash, using the
/// [`DefaultHasher`]. A `DefaultHasher` is unlikely to be the fastest option
/// and therefore the hash implementation can be specified by the user with
/// [`with_hasher`]:
///
/// ```rust
/// use fnv::FnvBuildHasher;
///
/// // Use the Fowler–Noll–Vo hash function - fantastic at very small keys, but
/// // there are better algorithms for larger keys!
/// let mut anchor = anchorhash::Builder::with_hasher(FnvBuildHasher::default()).build(50);
/// # anchor.add_resource(1);
/// # anchor.get_resource(1);
/// ```
///
/// [`with_resources`]: Self::with_resources  
/// [`with_hasher`]: Self::with_hasher  
/// [`DefaultHasher`]: std::collections::hash_map::DefaultHasher  
#[derive(Debug, Clone)]
pub struct Builder<R, B>
where
    B: BuildHasher,
{
    resources: Option<Vec<R>>,
    hasher: B,
}

/// Initialise an empty AnchorHash instance using the [`DefaultHasher`] and no
/// pre-populated resources.
///
/// [`DefaultHasher`]: std::collections::hash_map::DefaultHasher  
impl<R> Default for Builder<R, RandomState> {
    fn default() -> Self {
        Self {
            hasher: RandomState::default(),
            resources: None,
        }
    }
}

impl<R, B> Builder<R, B>
where
    B: BuildHasher,
{
    /// Initialise the `AnchroHash` instance with support for up to `capacity`
    /// number of resources.
    ///
    /// # Panics
    ///
    /// This method panics if the number of resources given to
    /// [`with_resources`] exceeds `capacity`.
    ///
    /// [`with_resources`]: Self::with_resources  
    pub fn build<K: Hash>(self, capacity: u16) -> AnchorHash<K, R, B> {
        let mut anchor = Anchor::new(capacity, 0);
        let mut resources = HashMap::new();

        if let Some(res) = self.resources {
            for r in res {
                let bucket = anchor
                    .add_bucket()
                    .expect("number of resources cannot exceed capacity");
                resources.insert(bucket, r);
            }
        }

        AnchorHash {
            anchor,
            hasher: self.hasher,
            resources,
            _key_type: PhantomData::default(),
        }
    }

    /// Use the provided hash algorithm when hashing keys.
    pub fn with_hasher(builder: B) -> Self {
        Self {
            hasher: builder,
            resources: None,
        }
    }

    /// Construct the `AnchorHash` with an initial set of resources.
    pub fn with_resources(self, resources: impl IntoIterator<Item = R>) -> Self {
        Self {
            resources: Some(resources.into_iter().collect()),
            ..self
        }
    }
}

impl<R, K> FromIterator<R> for AnchorHash<K, R, RandomState>
where
    K: Hash,
{
    fn from_iter<T: IntoIterator<Item = R>>(iter: T) -> Self {
        let resources = iter.into_iter().collect::<Vec<_>>();
        let n = u16::try_from(resources.len()).expect("too many resources");
        Builder::default().with_resources(resources).build(n)
    }
}

/// An `AnchorHash` instance consistently maps keys of type `K` to resources of
/// type `R` using the algorithm described in [`AnchorHash: A Scalable
/// Consistent Hash`].
///
/// The AnchorHash algorithm uniformly balances keys across all the configured
/// backends and performs optimal (minimal) rebalancing when existing backends
/// are removed or new backends added. AnchorHash achieves this using only a few
/// bytes per resource, and outperforms state-of-the-art algorithms.
///
/// # Hashing Algorithm
///
/// The hashing algorithm used by default is the same as the one used by Rust's
/// [`HashMap`] and can be [easily swapped] for a more performant hashing
/// algorithm.
///
/// AnchorHash does NOT require a cryptographic hash, but DOES require the hash
/// to produce uniformly distributed values.
///
/// # Distributed Consistency
///
/// In order for multiple AnchorHash instances to map the same keys to the same
/// resources, all instances must reach consensus on the ordering of changes to
/// the resource set.
///
/// # Key and Resource Types
///
/// Any type can be used as a resource type, including both owned any borrowed
/// content. Good examples include socket addresses, connection pools, API
/// clients, etc.
///
/// Any type that implements [`Hash`] can be used as a key. All primitive types
/// implement `Hash` (strings, `usize`, etc):
///
/// ```rust
/// # use anchorhash::AnchorHash;
/// #
/// // Build an AnchorHash from a list of backends.
/// //
/// // Backends can be added and removed, but this AnchorHash instance can hold
/// // a maximum of 2 backends when constructed by collecting an iterator
/// // (capacity == iterator length).
/// let anchor = vec!["cache1.itsallbroken.com", "cache2.itsallbroken.com"]
///     .into_iter()
///     .collect::<AnchorHash<_, _, _>>();
///
/// let backend_1 = anchor.get_resource("user-A").unwrap();
/// let backend_2 = anchor.get_resource("user-B").unwrap();
/// ```
///
/// You can also derive `Hash` on your types - this makes it easy to use a
/// compound key safely and without resorting to string types:
///
/// ```rust
/// use std::net::SocketAddr;
///
/// // A custom key type that maps to a backend based on all values
/// #[derive(Hash)]
/// struct UserSession {
///     user_id: u64,
///     ip_addr: SocketAddr,
/// }
///
/// // Initialise a AnchorHash with the capacity for 20 backend cache servers,
/// // and 3 active servers.
/// let anchor = anchorhash::Builder::default()
///     .with_resources(vec![
///         "cache1.itsallbroken.com",
///         "cache2.itsallbroken.com",
///         "cache3.itsallbroken.com",
///     ])
///     .build(20);
///
/// // Look up a cache backend for this user ID and requesting IP
/// let key = UserSession {
///     user_id: 42,
///     ip_addr: "127.0.0.1:4242".parse().unwrap(),
/// };
///
/// // Map the UserSession to a cache backend
/// let backend = anchor.get_resource(&key).unwrap();
///
/// println!("user mapped to: {}", backend);
/// ```
///
/// [`AnchorHash: A Scalable Consistent Hash`]: https://arxiv.org/abs/1812.09674
/// [`HashMap`]: std::collections::HashMap  
/// [easily swapped]: Builder::with_hasher  
/// [`Hash`]: std::hash::Hash  
#[derive(Debug)]
pub struct AnchorHash<K, R, B>
where
    K: Hash,
    B: BuildHasher,
{
    anchor: Anchor,
    hasher: B,
    resources: HashMap<u16, R>,

    _key_type: PhantomData<K>,
}

/// Implement `Clone` when both the resource type (`R`) and the hash builder
/// (`B`) implement clone.
///
/// Note the key type (`K`) does NOT have to implement `Clone`.
impl<K, R, B> Clone for AnchorHash<K, R, B>
where
    K: Hash,
    B: BuildHasher + Clone,
    R: Clone,
{
    fn clone(&self) -> Self {
        Self {
            anchor: self.anchor.clone(),
            hasher: self.hasher.clone(),
            resources: self.resources.clone(),
            _key_type: PhantomData::default(),
        }
    }
}

impl<K, R, B> AnchorHash<K, R, B>
where
    K: Hash,
    B: BuildHasher,
    R: PartialEq,
{
    /// Consistently hash `key` to a configured resource.
    pub fn get_resource(&self, key: K) -> Option<&R> {
        // Hash the key to a u32 value
        let mut hasher = self.hasher.build_hasher();
        key.hash(&mut hasher);
        let key = hasher.finish();

        // Lookup the bucket this key maps to
        let b = self.anchor.get_bucket(key as u32);

        // Resolve the bucket -> resource indirection
        self.resources.get(&b)
    }

    /// Add `resource`, allowing keys to map to it.
    ///
    /// When a new resource is added, keys immediately begin mapping to it, and
    /// the load across all the resources remains uniformly balanced. If there
    /// were 3 backends, each handling `1/3` of the load, adding a new resource
    /// (total: 4) means they all immediately become responsible for `1/4` the
    /// load.
    ///
    /// A subset of keys from each backend is mapped to the new resource
    /// ensuring minimal disruption with optimal load sharing.
    pub fn add_resource(&mut self, resource: R) -> Result<()> {
        let b = self
            .anchor
            .add_bucket()
            .ok_or(Error::CapacityLimitReached)?;

        // The bucket MUST NOT already be in use
        assert!(self.resources.insert(b, resource).is_none());

        Ok(())
    }

    /// Remove the resource, preventing keys from mapping to `resource`.
    ///
    /// When `resource` is removed, all the keys that previously mapped to it
    /// are uniformly distributed over the remaining backends. Keys that did not
    /// map to `resource` continue mapping to the same resource as before the
    /// removal.
    ///
    /// Removal runs in linear time w.r.t the number of resources.
    pub fn remove_resource(&mut self, resource: &R) -> Result<()> {
        // This could be an O(1) operation by using a bimap, but then R would
        // require Hash bounds making this implementation less flexible.
        //
        // In practice, a linear search appears 'good enough' from the benchmark
        // data - removing an element from a capacity=10000 & resources=1000
        // AnchorHash instance takes ~1us on a 2.6Ghz Intel Core i7.

        let b = self
            .resources
            .iter()
            .find(|(_k, r)| *r == resource)
            .map(|(&k, _r)| k)
            .ok_or(Error::ResourceNotFound)?;

        self.resources.remove(&b);
        self.anchor.remove_bucket(b);
        Ok(())
    }

    /// Returns an iterator yielding references to the configured resources in
    /// an arbitrary order.
    pub fn resources(&self) -> ResourceIterator<'_, R> {
        self.resources.values().into()
    }

    /// Returns an iterator yielding mutable references to the configured
    /// resources in an arbitrary order.
    pub fn resources_mut(&mut self) -> ResourceMutIterator<'_, R> {
        self.resources.values_mut().into()
    }
}

#[cfg(test)]
mod tests {
    use hashbrown::HashSet;

    use super::*;

    #[derive(Debug, PartialEq)]
    struct BackendServer {
        id: usize,
    }

    #[test]
    fn test_build_with_resources() {
        // Use a set of numbers as a dummy array of resources
        let servers = vec!["A", "B", "C", "D"];

        let a: AnchorHash<usize, _, _> =
            Builder::default().with_resources(servers.clone()).build(10);

        // Check a bucket was allocated for each resource
        let working = a.anchor.working_buckets();
        assert_eq!(working.len(), servers.len());

        // Check the resource map is fully populated
        assert_eq!(a.resources.len(), servers.len());

        // All the keys map to working buckets (and are distinct as the map
        // capacity matches the number of working buckets)
        for bucket in a.resources.keys() {
            assert!(working.contains(bucket));
        }

        // All the resources are present in the map
        let values = a.resources.values().cloned().collect::<HashSet<_>>();
        assert_eq!(values, servers.into_iter().collect::<HashSet<_>>());
    }

    #[test]
    fn test_anchorhash_borrowed_resources() {
        let server_a = BackendServer { id: 1 };
        let server_b = BackendServer { id: 2 };

        let mut a = Builder::default().build(20);

        a.add_resource(&server_a).unwrap();
        a.add_resource(&server_b).unwrap();

        // Ensure the key maps to one of the two servers
        let got = a.get_resource("a key").expect("should return a resource");
        assert!(dbg!(got) == &&server_a || got == &&server_b);

        // Remove server B and ensure the key maps to server A
        a.remove_resource(&&server_b)
            .expect("removing existing resource should succeed");
        let got = a.get_resource("a key").expect("should return a resource");
        assert_eq!(got, &&server_a);
    }

    #[test]
    fn test_anchorhash_owned_resources() {
        let server_a = BackendServer { id: 1 };
        let server_b = BackendServer { id: 2 };

        let mut a = Builder::default().build(20);

        a.add_resource(&server_a).unwrap();
        a.add_resource(&server_b).unwrap();

        // Ensure the key maps to one of the two servers
        let got = a.get_resource("a key").expect("should return a resource");
        assert!(dbg!(got) == &&server_a || got == &&server_b);

        // Remove server B and ensure the key maps to server A
        a.remove_resource(&&server_b)
            .expect("removing existing resource should succeed");
        let got = a.get_resource("a key").expect("should return a resource");
        assert_eq!(got, &&server_a);
    }

    #[test]
    fn test_full() {
        let mut a: AnchorHash<usize, _, _> = Builder::default().build(2);

        a.add_resource(1).unwrap();
        a.add_resource(2).unwrap();

        let err = a
            .add_resource(3)
            .expect_err("should not allow 3rd resource for capacity == 2");

        assert_eq!(err, Error::CapacityLimitReached);
    }

    #[test]
    fn test_remove_not_found() {
        let mut a: AnchorHash<usize, _, _> = Builder::default().build(2);

        a.add_resource(1).unwrap();
        a.add_resource(2).unwrap();

        let err = a
            .remove_resource(&3)
            .expect_err("should not allow removing non-existant resource");

        assert_eq!(err, Error::ResourceNotFound);
    }

    #[test]
    fn test_cloneable() {
        let mut a: AnchorHash<usize, _, _> = Builder::default().build(2);

        a.add_resource(1).unwrap();
        a.add_resource(2).unwrap();

        let b = a.clone();

        // Assert resources match
        let got_a = a.resources().cloned().collect::<HashSet<_>>();
        let got_b = b.resources().cloned().collect::<HashSet<_>>();
        assert_eq!(got_a, got_b);

        // Assert they match equally
        for i in 0..100 {
            assert_eq!(a.get_resource(i), b.get_resource(i));
        }
    }
}
