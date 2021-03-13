use std::iter::FusedIterator;

use hashbrown::hash_map::{Values, ValuesMut};

/// An iterator yielding resources assigned to an [`AnchorHash`] instance in
/// an arbitrary order.
///
/// [`AnchorHash`]: crate::AnchorHash  
#[derive(Debug, Clone)]
pub struct ResourceIterator<'a, R>(Values<'a, u16, R>);

impl<'a, R> From<Values<'a, u16, R>> for ResourceIterator<'a, R> {
    fn from(v: Values<'a, u16, R>) -> Self {
        Self(v)
    }
}

impl<'a, R> Iterator for ResourceIterator<'a, R> {
    type Item = &'a R;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<'a, R> ExactSizeIterator for ResourceIterator<'a, R> {}
impl<'a, R> FusedIterator for ResourceIterator<'a, R> {}

/// An iterator yielding mutable references to the resources assigned to an
/// [`AnchorHash`] instance in an arbitrary order.
///
/// [`AnchorHash`]: crate::AnchorHash  
#[derive(Debug)]
pub struct ResourceMutIterator<'a, R>(ValuesMut<'a, u16, R>);

impl<'a, R> From<ValuesMut<'a, u16, R>> for ResourceMutIterator<'a, R> {
    fn from(v: ValuesMut<'a, u16, R>) -> Self {
        Self(v)
    }
}

impl<'a, R> Iterator for ResourceMutIterator<'a, R> {
    type Item = &'a mut R;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<'a, R> ExactSizeIterator for ResourceMutIterator<'a, R> {}
impl<'a, R> FusedIterator for ResourceMutIterator<'a, R> {}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_fused_impl<F: FusedIterator>(_iter: F) {}

    #[test]
    fn test_iter_fused() {
        let a = vec!["A", "B", "C", "D"]
            .into_iter()
            .collect::<crate::AnchorHash<usize, _, _>>();

        assert_fused_impl(a.resources());
        assert_fused_impl(a.resources().0);
    }

    #[test]
    fn test_iter_mut_fused() {
        let mut a = vec!["A", "B", "C", "D"]
            .into_iter()
            .collect::<crate::AnchorHash<usize, _, _>>();

        assert_fused_impl(a.resources_mut());
        assert_fused_impl(a.resources_mut().0);
    }

    #[test]
    fn test_exact_size_iter() {
        let a = vec!["A", "B", "C", "D"]
            .into_iter()
            .collect::<crate::AnchorHash<usize, _, _>>();

        let got = a.resources().len();
        assert_eq!(got, 4);
    }

    #[test]
    fn test_exact_size_iter_mut() {
        let mut a = vec!["A", "B", "C", "D"]
            .into_iter()
            .collect::<crate::AnchorHash<usize, _, _>>();

        let got = a.resources_mut().len();
        assert_eq!(got, 4);
    }

    #[test]
    fn test_resource_iter() {
        let mut resources = vec!["A", "B", "C", "D"];
        let a = resources
            .clone()
            .into_iter()
            .collect::<crate::AnchorHash<usize, _, _>>();

        let mut got = a.resources().cloned().collect::<Vec<_>>();
        assert_eq!(resources.sort(), got.sort());
    }

    #[test]
    fn test_resource_iter_mut() {
        let mut resources = vec!["A", "B", "C", "D"];
        let mut a = resources
            .clone()
            .into_iter()
            .collect::<crate::AnchorHash<usize, _, _>>();

        let mut got = a.resources_mut().map(|v| v.clone()).collect::<Vec<_>>();
        assert_eq!(resources.sort(), got.sort());
    }
}
