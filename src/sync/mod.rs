use std::collections::VecDeque;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll, Waker};

use uuid::Uuid;

#[derive(Debug)]
pub struct TaggedMutex<K> {
    queue: Arc<parking_lot::Mutex<VecDeque<TaggedMutexQueueEntry<K>>>>
}

#[derive(Debug)]
pub struct TaggedMutexGuard<'m, K: PartialEq + std::fmt::Debug> {
    _handle: Arc<TaggedMutexHandle<'m, K>>
}

#[derive(Debug)]
struct TaggedMutexQueueEntry<K> {
    id: Uuid,
    key: K,
    waker: Waker
}

#[derive(Debug)]
struct TaggedMutexHandle<'m, K: PartialEq + std::fmt::Debug> {
    id: Uuid,
    key: K,
    mutex: &'m TaggedMutex<K>
}

#[derive(Debug)]
struct TaggedMutexFuture<'m, K: PartialEq + std::fmt::Debug> {
    handle: Arc<TaggedMutexHandle<'m, K>>
}

impl<K: PartialEq + Clone + std::fmt::Debug> TaggedMutex<K> {
    pub fn new() -> TaggedMutex<K> {
        TaggedMutex {
            queue: Arc::new(parking_lot::Mutex::new(VecDeque::new()))
        }
    }

    pub async fn lock(&self, key: K) -> TaggedMutexGuard<K> {
        TaggedMutexFuture {
            handle: Arc::new(TaggedMutexHandle {
                id: Uuid::new_v4(),
                key,
                mutex: self
            })
        }
        .await
    }
}

impl<'m, K: PartialEq + std::fmt::Debug> Drop for TaggedMutexHandle<'m, K> {
    fn drop(&mut self) {
        let mut queue = self.mutex.queue.lock();
        let (ref_entry_index, _) = queue
            .iter()
            .enumerate()
            .find(|(_, el)| el.id == self.id)
            .expect("guard id inserted by lock function");
        queue.remove(ref_entry_index);
        if let Some(next_entry) = queue.iter_mut().find(|el| el.key == self.key) {
            next_entry.waker.clone().wake();
        }
    }
}

impl<'m, K: PartialEq + Clone + std::fmt::Debug> Future for TaggedMutexFuture<'m, K> {
    type Output = TaggedMutexGuard<'m, K>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut queue = self.handle.mutex.queue.lock();
        match queue.iter().find(|el| el.key == self.handle.key) {
            Some(TaggedMutexQueueEntry { id, .. }) => match self.handle.id == *id {
                false => {
                    match queue.iter_mut().find(|el| el.id == self.handle.id) {
                        None => queue.push_back(TaggedMutexQueueEntry {
                            id: self.handle.id,
                            key: self.handle.key.clone(),
                            waker: cx.waker().clone()
                        }),
                        Some(el) => el.waker = cx.waker().clone()
                    }
                    Poll::Pending
                }
                true => Poll::Ready(TaggedMutexGuard {
                    _handle: self.handle.clone()
                })
            },
            None => {
                queue.push_back(TaggedMutexQueueEntry {
                    id: self.handle.id,
                    key: self.handle.key.clone(),
                    waker: cx.waker().clone()
                });
                Poll::Ready(TaggedMutexGuard {
                    _handle: self.handle.clone()
                })
            }
        }
    }
}
