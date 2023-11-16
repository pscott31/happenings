use indexmap::IndexMap;
use leptos::*;
use uuid::Uuid;

pub type ReactiveList<T> = IndexMap<Uuid, RwSignal<T>>;

pub trait TrackableList<T> {
    fn tracked_push(&self, guest: T);
    fn tracked_remove(&self, uid: Uuid);
    fn tracked_insert(&self, uid: Uuid, new: T);
}

impl<S, T> TrackableList<T> for S
where
    S: SignalUpdate<Value = ReactiveList<T>>,
    T: 'static,
{
    fn tracked_push(&self, guest: T) {
        self.update(|gs| {
            gs.insert(Uuid::new_v4(), create_rw_signal::<T>(guest));
        });
    }

    fn tracked_remove(&self, uid: Uuid) {
        self.update(|gs| {
            gs.shift_remove(&uid);
        });
    }

    fn tracked_insert(&self, uid: Uuid, new: T) {
        self.update(|gs| {
            gs.insert(uid, create_rw_signal::<T>(new));
        });
    }
}

