#[derive(Clone, Debug)]
/// Auth struct, used to authenticate with Azure Speech Services.
pub struct Auth {
    pub(crate) region: String,
    pub(crate) subscription: String,
}
impl Auth {
    /// Create a new Auth instance from a subscription key and a region.
    pub fn from_subscription(region: impl Into<String>, subscription: impl Into<String>) -> Self {
        Auth {
            region: region.into(),
            subscription: subscription.into(),
        }
    }
}
