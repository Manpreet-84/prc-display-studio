//! Page ids in spec order (PAGE 01..16) with wrap-around navigation.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageId {
    Main,
    Central,
    Performance,
    System,
    Media,
    Network,
    Servers,
    Temperature,
    Cooling,
    Storage,
    Clock,
    Terminal,
    Art,
    Cyberpunk,
    Minimal,
    Alert,
}

impl PageId {
    pub const ALL: [PageId; 16] = [
        PageId::Main,
        PageId::Central,
        PageId::Performance,
        PageId::System,
        PageId::Media,
        PageId::Network,
        PageId::Servers,
        PageId::Temperature,
        PageId::Cooling,
        PageId::Storage,
        PageId::Clock,
        PageId::Terminal,
        PageId::Art,
        PageId::Cyberpunk,
        PageId::Minimal,
        PageId::Alert,
    ];

    pub fn id(&self) -> &'static str {
        match self {
            PageId::Main => "main",
            PageId::Central => "central",
            PageId::Performance => "performance",
            PageId::System => "system",
            PageId::Media => "media",
            PageId::Network => "network",
            PageId::Servers => "servers",
            PageId::Temperature => "temperature",
            PageId::Cooling => "cooling",
            PageId::Storage => "storage",
            PageId::Clock => "clock",
            PageId::Terminal => "terminal",
            PageId::Art => "art",
            PageId::Cyberpunk => "cyberpunk",
            PageId::Minimal => "minimal",
            PageId::Alert => "alert",
        }
    }

    pub fn from_id(id: &str) -> Option<PageId> {
        PageId::ALL.iter().copied().find(|p| p.id() == id)
    }

    pub fn index(&self) -> usize {
        PageId::ALL.iter().position(|p| p == self).unwrap_or(0)
    }

    /// Alt+PageDown — wrap from last to first.
    pub fn next(self) -> PageId {
        PageId::ALL[(self.index() + 1) % PageId::ALL.len()]
    }

    /// Alt+PageUp — wrap from first to last.
    pub fn prev(self) -> PageId {
        PageId::ALL[(self.index() + PageId::ALL.len() - 1) % PageId::ALL.len()]
    }
}
