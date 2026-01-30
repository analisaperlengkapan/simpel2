// Copyright 2025 Secreton Security Engine System Contributors
// SPDX-License-Identifier: Apache-2.0

//! FIPS Compliance Module
//!
//! Provides types and traits for managing FIPS 140 compliance levels.

use serde::{Deserialize, Serialize};

/// FIPS 140 Compliance Levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub enum FipsLevel {
    /// Not FIPS compliant
    #[default]
    None,
    /// FIPS 140-2 Level 1
    Fips140_2Level1,
    /// FIPS 140-2 Level 2
    Fips140_2Level2,
    /// FIPS 140-2 Level 3
    Fips140_2Level3,
    /// FIPS 140-2 Level 4
    Fips140_2Level4,
    /// FIPS 140-3 Level 1
    Fips140_3Level1,
    /// FIPS 140-3 Level 2
    Fips140_3Level2,
    /// FIPS 140-3 Level 3
    Fips140_3Level3,
    /// FIPS 140-3 Level 4
    Fips140_3Level4,
}

/// Trait for components that can report their FIPS compliance level
pub trait FipsCompliant {
    /// Get the FIPS compliance level of the component
    fn fips_level(&self) -> Option<FipsLevel>;

    /// Check if the component is FIPS compliant (any level)
    fn is_fips_compliant(&self) -> bool {
        self.fips_level().is_some() && self.fips_level() != Some(FipsLevel::None)
    }
}
