pub enum Currency {
    Usdc,
    SepoliaEth,
    GoerliEth,
    LocalEth,
}

impl Currency {
    pub fn from_str(s: String) -> Option<Currency> {
        match s.to_lowercase().as_str() {
            "usdc" => Some(Currency::Usdc),
            "sepoliaeth" => Some(Currency::SepoliaEth),
            "goerlieth" => Some(Currency::GoerliEth),
            "localeth" => Some(Currency::LocalEth),
            _ => None,
        }
    }
}
