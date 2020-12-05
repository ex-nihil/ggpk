#[derive(Clone, Copy)]
pub struct GGPKVersion {
    id: u32,
    utf32_paths: bool,
}

pub const GGPK_VERSIONS: [GGPKVersion; 4] = [
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
    GGPKVersion {
        id: 1,
        utf32_paths: false,
    },
];

impl GGPKVersion {
    pub fn from_id(id: u32) -> GGPKVersion {
        let version = GGPK_VERSIONS.iter().find(|v| v.id == id);
        match version {
            Some(&v) => v,
            None => {
                warn!("GGPK has unknown version. Using the latest known.");
                GGPK_VERSIONS[0]
            }
        }
    }
}

pub trait GGPKVersionImpl {
    fn use_utf32(&self) -> bool;
}

impl GGPKVersionImpl for GGPKVersion {
    fn use_utf32(&self) -> bool {
        self.utf32_paths
    }
}