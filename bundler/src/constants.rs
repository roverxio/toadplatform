pub struct Constants;

impl Constants {
    // Users
    pub const ADMIN: &'static str = "admin";

    // entity
    pub const PAYMASTER: &'static str = "paymaster";
    pub const RELAYER: &'static str = "relayer";
    pub const VERIFYING_PAYMASTER: &'static str = "verifying";

    // Currency
    pub const NATIVE: &'static str = "native";

    // User Operation Event signature
    pub const USER_OPERATION_EVENT: &'static str =
        "0x49628fd1471006c1482da88028e9ce4dbb080b815c9b0344d39e5a8e6ec1419f";
}
