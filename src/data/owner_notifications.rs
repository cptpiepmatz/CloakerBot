#[derive(Debug, Clone, Default)]
pub struct OwnerNotified {
    pub allow_dms: Option<bool>,
    pub latest_msg_index: usize
}
