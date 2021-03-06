#[derive(Clone, Copy)]
pub struct GGPKVersion {
    id: u32,
    utf32_paths: bool,
}

pub const GGPK_VERSIONS: [GGPKVersion; 3] = [
    GGPKVersion {
        id: 4,
        utf32_paths: true,
    },
    GGPKVersion {
        id: 3,
        utf32_paths: false,
    },
    GGPKVersion {
        id: 2,
        utf32_paths: false,
    },
];

impl GGPKVersion {
    pub fn from_id(id: u32) -> GGPKVersion {
        info!("GGPK version: {}", id);
        match GGPK_VERSIONS.iter().find(|v| v.id == id) {
            Some(&version) => version,
            None => {
                warn!("Unknown GGPK version. Using latest known implementation.");
                GGPK_VERSIONS[0]
            }
        }
    }

    pub fn use_utf32(&self) -> bool {
        self.utf32_paths
    }
}
