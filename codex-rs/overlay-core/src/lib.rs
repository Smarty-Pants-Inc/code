//! Shared overlay utilities for Smarty's SDK.
//! Currently a thin wrapper so that future customizations live outside
//! upstream-modified modules.

pub mod protocol {
    use codex_core::protocol::{Event, EventMsg, SandboxPolicy};

    /// Extension trait for sandbox metadata. Keeps logic separate from
    /// upstream enums so future patches stay localized.
    pub trait SandboxPolicyExt {
        fn network_access_enabled(&self) -> bool;
    }

    impl SandboxPolicyExt for SandboxPolicy {
        fn network_access_enabled(&self) -> bool {
            matches!(
                self,
                SandboxPolicy::DangerFullAccess
                    | SandboxPolicy::WorkspaceWrite {
                        network_access: true,
                        ..
                    }
            )
        }
    }

    /// Light-weight view over upstream events. Overlays can attach additional
    /// metadata while preserving the original struct.
    #[derive(Debug, Clone)]
    pub struct EventView<'a> {
        pub id: &'a str,
        pub msg: &'a EventMsg,
    }

    impl<'a> From<&'a Event> for EventView<'a> {
        fn from(event: &'a Event) -> Self {
            Self {
                id: &event.id,
                msg: &event.msg,
            }
        }
    }
}

pub mod tui {
    /// Placeholder for future TUI overlays. By centralizing extension points
    /// we can evolve branding/theme tweaks without touching upstream files.
    pub trait IntroBannerProvider {
        fn banner_lines(&self) -> &'static [&'static str];
    }

    /// Default implementation retains existing behavior; overrides can be
    /// supplied by the Smarty runtime.
    pub struct DefaultBanner;

    impl IntroBannerProvider for DefaultBanner {
        fn banner_lines(&self) -> &'static [&'static str] {
            &["Welcome to Smarty"]
        }
    }
}
