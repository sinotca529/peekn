use std::marker::PhantomData;

#[derive(Clone, Hash, Debug)]
pub struct PeekNIterator<I: Iterator, const N: usize> {
    iter: I,
    queue: [Option<I::Item>; N],
    /// Index of self.peeked to get next an element.
    next_idx: usize,
    /// Number of currently peeked elements.
    num_peeked: usize,
}

impl<I: Iterator, const N: usize> PeekNIterator<I, N> {
    fn new(iter: I) -> Self {
        struct Helper<I: Iterator, const N: usize>(PhantomData<fn() -> I>);
        impl<I: Iterator, const N: usize> Helper<I, N> {
            const OPT_NONE: Option<I::Item> = None;
        }
        Self {
            iter,
            queue: [Helper::<I, N>::OPT_NONE; N],
            next_idx: 0,
            num_peeked: 0,
        }
    }

    pub fn peek_nth(&mut self, n: usize) -> Option<&I::Item> {
        assert!(n < N);
        if self.num_peeked <= n {
            for i in (self.next_idx + self.num_peeked)..=(self.next_idx + n) {
                self.queue[i % N] = self.iter.next();
            }
            self.num_peeked = n + 1;
        }
        self.queue[(self.next_idx + n) % N].as_ref()
    }
}

impl<I: Iterator, const N: usize> Iterator for PeekNIterator<I, N> {
    type Item = I::Item;

    #[inline]
    fn next(&mut self) -> Option<I::Item> {
        if self.num_peeked == 0 {
            self.iter.next()
        } else {
            let ret = self.queue[self.next_idx].take();
            self.num_peeked -= 1;
            self.next_idx = (self.next_idx + 1) % N;
            ret
        }
    }

    #[inline]
    fn count(self) -> usize {
        let mut c = self.iter.count();
        for i in self.next_idx..(self.next_idx + self.num_peeked) {
            if self.queue[i % N].is_some() {
                c += 1;
            } else {
                return c;
            }
        }
        return c;
    }

    fn nth(&mut self, n: usize) -> Option<I::Item> {
        if n < self.num_peeked {
            for i in self.next_idx..(self.next_idx + n) {
                self.queue[i % N] = None;
            }
            let ret = self.queue[(self.next_idx + n) % N].take();
            self.num_peeked -= n + 1;
            self.next_idx = (self.next_idx + n + 1) % N;
            ret
        } else {
            for i in self.next_idx..(self.next_idx + self.num_peeked) {
                self.queue[i % N] = None;
            }
            self.num_peeked = 0;
            self.next_idx = (self.next_idx + self.num_peeked) % N;
            self.iter.nth(n - self.num_peeked)
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (lo, hi) = self.iter.size_hint();
        let lo = lo.saturating_add(self.num_peeked);
        let hi = hi.and_then(|e| e.checked_add(self.num_peeked));
        (lo, hi)
    }
}

impl<I: ExactSizeIterator, const N: usize> ExactSizeIterator for PeekNIterator<I, N> {}

pub trait PeekN: Iterator + Sized {
    fn peekn<const N: usize>(self) -> PeekNIterator<Self, N>;
}

impl<I: Iterator> PeekN for I {
    fn peekn<const N: usize>(self) -> PeekNIterator<Self, N> {
        PeekNIterator::<I, N>::new(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_peek_nth() {
        let v = vec![0, 1, 2, 3];
        let mut p = v.into_iter().peekn::<3>();
        assert_eq!(p.peek_nth(0), Some(&0));
        assert_eq!(p.peek_nth(1), Some(&1));
        assert_eq!(p.peek_nth(2), Some(&2));
        assert_eq!(p.next(), Some(0));
        assert_eq!(p.next(), Some(1));
        assert_eq!(p.peek_nth(1), Some(&3));
        assert_eq!(p.next(), Some(2));
        assert_eq!(p.next(), Some(3));
        assert_eq!(p.next(), None);
    }

    #[test]
    fn test_nth() {
        let v = vec![0, 1, 2, 3];
        let mut p = v.into_iter().peekn::<3>();
        assert_eq!(p.peek_nth(2), Some(&2));
        assert_eq!(p.nth(1), Some(1));
    }

    #[test]
    fn test_count() {
        let v = vec![0, 1, 2, 3];
        let mut p = v.into_iter().peekn::<10>();
        assert_eq!(p.peek_nth(9), None);
        assert_eq!(p.count(), 0);
    }
}
