use twilight_model::id::marker::UserMarker;
use twilight_model::id::Id;

// TODO: store which owners got notified and if they allow dms

pub struct OwnerNotified {
    owner_id: Id<UserMarker>
}
