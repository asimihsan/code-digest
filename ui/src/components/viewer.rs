/*
 * Copyright (c) 2025 Asim Ihsan.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * SPDX-License-Identifier: MPL-2.0
 */

use dioxus::prelude::*;

#[component]
pub fn Viewer() -> Element {
    rsx! {
        div { id: "viewer",
            // The viewer will be populated here
            "Viewer"
        }
    }
}
