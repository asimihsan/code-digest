/*
 * Copyright (c) 2023 Asim Ihsan.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * SPDX-License-Identifier: MPL-2.0
 */

#[path = "static_metrics_extract.rs"]
mod extract;
#[path = "static_metrics_support.rs"]
mod support;

pub use extract::extract_static_metrics_observations;
pub use support::{
    supports_static_metrics, MetricExtraction, StaticMetricsLanguageSupport,
    FIRST_SUPPORTED_STATIC_METRICS_LANGUAGES,
};
