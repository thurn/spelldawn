// Copyright © Spelldawn 2021-present

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at

//    https://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use core_ui::prelude::*;
use core_ui::style;
use protos::spelldawn::{FlexPosition, ImageScaleMode, SpriteAddress};

pub struct AdventureLoading {
    image: SpriteAddress,
}

impl AdventureLoading {
    pub fn new(image: impl Into<String>) -> Self {
        Self { image: style::sprite(image.into()) }
    }
}

impl Component for AdventureLoading {
    fn build(self) -> Option<Node> {
        Row::new("AdventureLoading")
            .style(
                Style::new()
                    .position_type(FlexPosition::Absolute)
                    .position(Edge::All, 0.px())
                    .background_image(self.image)
                    .background_image_scale_mode(ImageScaleMode::ScaleAndCrop),
            )
            .build()
    }
}
