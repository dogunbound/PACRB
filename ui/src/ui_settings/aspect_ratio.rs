use crate::ui_settings::DEFAULT_UI_SETTING_OPTIONS;
use serde::{Deserialize, Serialize};
use sfml::system::Vector2;
use std::{collections::LinkedList, error::Error, str::FromStr};
use utils::simple_error::SimpleError;

macro_rules! const_ratio {
    ($aspect_ratio:expr, $base_resolution:expr) => {
        AspectRatio {
            aspect_ratio: Vector2::new($aspect_ratio.0, $aspect_ratio.1),
            base_resolution: Vector2::new($base_resolution.0, $base_resolution.1),
            computed_resolution: Vector2::new(0., 0.),
            current_resolution: Vector2::new(0., 0.),
        }
    };
}

pub const NUMBER_OF_DEFAULT_ASPECT_RATIOS: usize = 2;
pub(super) const DEFAULT_ASPECT_RATIOS: [AspectRatio; NUMBER_OF_DEFAULT_ASPECT_RATIOS] = [
    const_ratio!((16., 9.), (1024., 576.)),
    const_ratio!((4., 3.), (1024., 768.)),
];

// Look into docs/UI/scaling.lorien for an explanation
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct AspectRatio {
    aspect_ratio: Vector2<f32>,
    base_resolution: Vector2<f32>,
    computed_resolution: Vector2<f32>,
    pub current_resolution: Vector2<f32>,
}

impl AspectRatio {
    pub fn new(
        aspect_ratio: Vector2<f32>,
        base_resolution: Vector2<f32>,
    ) -> Result<Self, SimpleError> {
        if base_resolution.y * aspect_ratio.x / aspect_ratio.y != base_resolution.x {
            return Err(SimpleError::new(
                "Base_resolution's aspect_ratio does not match given aspect_ratio".to_string(),
            ));
        }

        Ok(Self {
            aspect_ratio,
            base_resolution,
            current_resolution: base_resolution,
            computed_resolution: base_resolution,
        })
    }

    pub fn aspect_ratio(&self) -> Vector2<f32> {
        self.aspect_ratio
    }

    pub fn base_resolution(&self) -> Vector2<f32> {
        self.base_resolution
    }

    pub fn computed_resolution(&self) -> Vector2<f32> {
        self.computed_resolution
    }

    pub fn compute_resolution(&mut self) {
        let mut smallest_base_resolution = self.aspect_ratio;
        while smallest_base_resolution.x < self.current_resolution.x
            && smallest_base_resolution.y < self.current_resolution.y
        {
            smallest_base_resolution += self.aspect_ratio;
        }
        let ratio = self.base_resolution.cwise_div(smallest_base_resolution);

        self.computed_resolution = self.current_resolution.cwise_mul(ratio);
    }

    pub fn relative_mouse_coords(&self, mouse_pos: Vector2<i32>) -> Vector2<i32> {
        Vector2::new(
            (self.computed_resolution.x / self.current_resolution.x) * mouse_pos.x as f32,
            (self.computed_resolution.y / self.current_resolution.y) * mouse_pos.y as f32,
        )
        .as_other()
    }
}

impl ToString for AspectRatio {
    fn to_string(&self) -> String {
        format!("{}:{}", self.aspect_ratio.x, self.aspect_ratio.y)
    }
}

impl FromStr for AspectRatio {
    type Err = Box<dyn Error>;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        let strings: Vec<&str> = string.split(':').collect();

        if strings.len() != 2 {
            return Err(Box::new(SimpleError::new(format!(
                "{:#?} may not be an aspect ratio string",
                string
            ))));
        } else if let (Ok(x), Ok(y)) = (strings[0].parse::<f32>(), strings[1].parse::<f32>()) {
            let potential_settings = DEFAULT_UI_SETTING_OPTIONS;
            for potential_aspect_ratio in potential_settings.aspect_ratios.iter() {
                if Vector2::new(x, y) == potential_aspect_ratio.aspect_ratio {
                    return Ok(*potential_aspect_ratio);
                }
            }
        }
        Err(Box::new(SimpleError::new(format!(
            "{:#?} aspect ratio does not exist!",
            string
        ))))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn new() {
        // Test failing aspect ratio
        assert!(AspectRatio::new(Vector2::new(16., 9.), Vector2::new(213., 3234.)).is_err());
    }

    #[test]
    fn to_from_ar_string() {
        let potential_aspect_ratios = DEFAULT_UI_SETTING_OPTIONS.aspect_ratios;

        for potential_aspect_ratio in potential_aspect_ratios.iter() {
            assert!(AspectRatio::from_str(&potential_aspect_ratio.to_string()).is_ok());
        }

        // failing try_from_ar_string
        assert!(AspectRatio::from_str("dddvfgbdas;kj").is_err());
        assert!(AspectRatio::from_str("99999:99999").is_err());
        assert!(AspectRatio::from_str("16:9:16:9").is_err());
    }

    #[test]
    fn compute_resolution() {
        let mut ar = AspectRatio::new(Vector2::new(16., 9.), Vector2::new(1280., 720.)).unwrap();
        ar.current_resolution = Vector2::new(2180., 1320.);
        ar.compute_resolution();
        assert_eq!(ar.computed_resolution, Vector2::new(1272.9927, 770.80286));
    }

    #[test]
    fn relative_mouse_coords() {
        let mut ar = AspectRatio::new(Vector2::new(16., 9.), Vector2::new(1280., 720.)).unwrap();
        ar.current_resolution = Vector2::new(2180., 1320.);
        ar.computed_resolution();

        let mouse_pos = Vector2::new(0, 0);
        assert_eq!(ar.relative_mouse_coords(mouse_pos), mouse_pos);
        let mouse_pos = ar.current_resolution.as_other();
        assert_eq!(
            ar.relative_mouse_coords(mouse_pos),
            ar.base_resolution().as_other()
        );
        let mouse_pos = ar.current_resolution.as_other::<i32>() / 2;
        assert_eq!(
            ar.relative_mouse_coords(mouse_pos),
            (ar.base_resolution() / 2.).as_other()
        );
    }
}
