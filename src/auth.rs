
#[derive(Clone, Debug)]
pub struct Auth {
    pub region: String,
    pub subscription: String,
}
impl Auth {
    pub fn from_subscription(region: impl Into<String>, subscription: impl Into<String>) -> Self {
        Auth {
            region: region.into(),
            subscription: subscription.into(),
        }
    }
    
    
}