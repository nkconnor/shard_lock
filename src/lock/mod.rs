#[cfg(feature = "lock-crossbeam")]
mod cross;

#[cfg(feature = "lock-parking-lot")]
mod parking;

use crate::shard::{index, Shard};
use crate::*;
use std::hash::Hash;

#[cfg(feature = "lock-parking-lot")]
pub type RwLock<T> = parking_lot::RwLock<T>;

#[cfg(feature = "lock-crossbeam")]
pub type RwLock<T> = crossbeam::sync::ShardedLock<T>;

#[cfg(not(any(feature = "lock-parking-lot", feature = "lock-crossbeam")))]
pub type RwLock<T> = std::sync::RwLock<T>;

use std::sync::{RwLock as StdRwLock, RwLockReadGuard, RwLockWriteGuard};

/// Generic locking implementation.
pub trait Lock<T> {
    #[rustfmt::skip]
    type ReadGuard<'a> where T: 'a;
    #[rustfmt::skip]
    type WriteGuard<'a> where T: 'a + std::ops::Deref<Target=T>;

    fn new(t: T) -> Self;

    fn write(&self) -> Self::WriteGuard<'_>;

    fn read(&self) -> Self::ReadGuard<'_>;
}

pub trait ShardLock<K: Hash, V, U, L>
where
    V: ExtractShardKey<K>,
    U: Collection<K, V>,
    L: Lock<U>,
{
    fn shards<'a>(&'a self) -> &'a [L];
    fn write(&self, k: &K) -> L::WriteGuard<'_>;
    fn read(&self, k: &K) -> L::ReadGuard<'_>;
}

impl<K: Hash, V, U, L> ShardLock<K, V, U, L> for Shard<L>
where
    V: ExtractShardKey<K>,
    U: Collection<K, V>,
    L: Lock<U>,
{
    fn shards<'a>(&'a self) -> &'a [L] {
        &self.shards
    }

    fn write(&self, k: &K) -> L::WriteGuard<'_> {
        let i = index(k);
        if let Some(lock) = self.shards.get(i) {
            lock.write()
        } else {
            panic!("index out of bounds")
        }
    }

    fn read(&self, k: &K) -> L::ReadGuard<'_> {
        let i = index(k);
        if let Some(lock) = self.shards.get(i) {
            lock.read()
        } else {
            panic!("index out of bounds")
        }
    }
}

impl<T> Lock<T> for StdRwLock<T> {
    #[rustfmt::skip]
    type ReadGuard<'b> where T: 'b = RwLockReadGuard<'b, T>;
    #[rustfmt::skip]
    type WriteGuard<'b> where T: 'b = RwLockWriteGuard<'b, T>;

    fn new(t: T) -> Self {
        StdRwLock::new(t)
    }

    fn read(&self) -> Self::ReadGuard<'_> {
        self.read().unwrap()
    }

    fn write(&self) -> Self::WriteGuard<'_> {
        self.write().unwrap()
    }
}

//impl<T> Shard<StdRwLock<T>> {
//    pub fn write<K: Hash>(&self, k: &K) -> RwLockWriteGuard<'_, T> {
//        let i = index(k);
//        self.shards
//            .get(i)
//            .map(|lock| lock.write().unwrap())
//            .unwrap()
//    }
//
//    pub fn read<K: Hash>(&self, k: &K) -> RwLockReadGuard<'_, T> {
//        let i = index(k);
//        self.shards.get(i).map(|lock| lock.read().unwrap()).unwrap()
//    }
//}
