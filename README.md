[![crates.io](https://img.shields.io/crates/v/anchorhash.svg)](https://crates.io/crates/anchorhash)
[![docs.rs](https://docs.rs/anchorhash/badge.svg)](https://docs.rs/anchorhash)

# AnchorHash

A consistent hashing algorithm described in [AnchorHash: A Scalable Consistent
Hash].

> Consistent hashing (CH) is a central building block in many networking
> applications, from datacenter load-balancing to distributed storage.
> Unfortunately, state-of-the-art CH solutions cannot ensure full consistency
> under arbitrary changes and/or cannot scale while maintaining reasonable
> memory footprints and update times. We present AnchorHash, a scalable and
> fully-consistent hashing algorithm. AnchorHash achieves high key lookup rates,
> a low memory footprint, and low update times. We formally establish its strong
> theoretical guarantees, and present advanced implementations with a memory
> footprint of only a few bytes per resource. Moreover, extensive evaluations
> indicate that it outperforms state-of-the-art algorithms, and that it can
> scale on a single core to 100 million resources while still achieving a key
> lookup rate of more than 15 million keys per second.

Key points:
* Uniform balancing of load across all resources
* **Optimal rebalancing of keys** when resources are added & removed
* Small memory footprint
* It's really, really fast!

`AnchorHash` consistently hashes keys onto resources under arbitrary working set
changes. It does this with a low memory footprint, fast key lookups (10s to 100s
of millions of lookups per second), optimal disruption and uniform balancing of
load across resources.

## Optimisations

This implementation makes use of SSE4.2 instructions by default on `x86_64`
platforms to quickly perform internal bucket hashing - the [Fowler–Noll–Vo hash]
is used as a fallback. The SIMD optimised hash can be manually disabled by
opting out of the `simd` crate feature. 

This implementation also makes use of Daniel Lemire's fast range mapping
algorithm presented in [Fast Random Integer Generation in an Interval] when
compiled on 64-bit architectures. This can be manually disabled by opting out of
the `fastmod` crate feature.

This implementation uses 16-bit integers to maximise cache locality, providing a
significant speed up for small capacity instances. This limits the total number
of addressable resources to 65,535.

## Benchmarks

Benchmarks that cover the hash algorithms, range mapping optimisations and
overall AnchorHash implementation are included in this crate - `cargo bench`
runs them.

[AnchorHash: A Scalable Consistent Hash]: https://arxiv.org/abs/1812.09674
[Fast Random Integer Generation in an Interval]: https://arxiv.org/abs/1805.10941
[Fowler–Noll–Vo hash]: http://www.isthe.com/chongo/tech/comp/fnv/index.html