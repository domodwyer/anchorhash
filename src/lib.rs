//! A consistent hashing algorithm that outperforms state-of-the-art algorithms.
//!
//! This crate provides an implementation of the algorithm described in
//! [AnchorHash: A Scalable Consistent Hash].
//!
//! [`AnchorHash`] consistently hashes keys onto resources under arbitrary
//! working set changes. It does this with a low memory footprint, fast key
//! lookups (10s to 100s of millions of lookups per second), optimal disruption
//! and uniform balancing of load across resources.
//!
//! This implementation also makes use of Daniel Lemire's fast range mapping
//! algorithm presented in [Fast Random Integer Generation in an Interval].
//!
//! # Example
//!
//! ```rust
//! // Initialise a AnchorHash with the capacity for 20 backend cache servers,
//! // and 3 active servers.
//! let anchor = anchorhash::Builder::default()
//!     .with_resources(vec![
//!         "cache1.itsallbroken.com",
//!         "cache2.itsallbroken.com",
//!         "cache3.itsallbroken.com",
//!     ])
//!     .build(20);
//!
//! // Map an input key to one of the backends
//! let backend = anchor.get_resource("user-A").unwrap();
//!
//! println!("user mapped to: {}", backend);
//! ```
//!
//! # Features
//!
//! This crate has several compile-time features:
//!
//! * `simd`: use SIMD operations to hash data internally (enabled by default on
//!   `x86_64` platforms with support for SSE4.2)
//! * `fastmod`: efficient range mapping from [Fast Random Integer Generation in
//!   an Interval] (enabled by default on 64-bit platforms)
//!
//! [AnchorHash: A Scalable Consistent Hash]: https://arxiv.org/abs/1812.09674  
//! [`AnchorHash`]: crate::AnchorHash
//! [Fast Random Integer Generation in an Interval]: https://arxiv.org/abs/1805.10941  

//   Copyright 2021 Dominic Dwyer (dom@itsallbroken.com)
//
//   Licensed under the Apache License, Version 2.0 (the "License");
//   you may not use this file except in compliance with the License.
//   You may obtain a copy of the License at
//
//       http://www.apache.org/licenses/LICENSE-2.0
//
//   Unless required by applicable law or agreed to in writing, software
//   distributed under the License is distributed on an "AS IS" BASIS,
//   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//   See the License for the specific language governing permissions and
//   limitations under the License.

#![deny(rust_2018_idioms, missing_debug_implementations, unreachable_pub)]
#![warn(
    missing_docs,
    clippy::todo,
    clippy::dbg_macro,
    clippy::clone_on_ref_ptr
)]
#![allow(clippy::missing_docs_in_private_items)]

mod anchor;

mod anchor_hash;
pub use anchor_hash::*;

mod range_map;
pub use range_map::*;

mod fasthash;
pub use fasthash::*;

mod iter;
pub use iter::*;
