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
pub fn FileTree() -> Element {
    rsx! {
        div { id: "filetree",
            // The file tree will be populated here
            "File Tree"
        }
    }
}
