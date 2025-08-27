use chrono::NaiveDateTime;
use serde::Serialize;
use sqlx::{FromRow, query_as};
use uuid::Uuid;

use crate::db;
use crate::utils::error::ModelResult;

/// Represents a pixel entry in the database
#[derive(FromRow)]
pub struct PixelModel {
    /// X coordinate of the pixel
    x: i64,

    /// Y coordinate of the pixel
    y: i64,

    /// ID of the user who placed the pixel
    user_id: Uuid,

    /// Color of the pixel as a 32-bit integer (hexadecimal representation)
    color: i32,

    /// Timestamp when the pixel was placed
    placed_at: NaiveDateTime,
}

/// Represents the data required to create a new pixel entry
#[derive(Deserialize)]
pub struct PixelCreation {
    /// A list of pixel data where:
    /// - First element is the x coordinate
    /// - Second element is the y coordinate
    /// - Third element is the color as a 32-bit integer (hexadecimal representation)
    pixels: Vec<[i64; 3]>,

    /// ID of the user who is creating the pixel entry (as a string)
    user_id: String,
}

/// Represents the result of a pixel operation
#[derive(Serialize)]
pub struct PixelResult {
    /// X coordinate of the pixel
    x: i64,

    /// Y coordinate of the pixel
    y: i64,

    /// uuid of the user who placed the pixel (as a string)
    user_id: String,

    /// Color of the pixel as a 32-bit integer (hexadecimal representation)
    color: i32,
}

impl PixelModel {
    /// Creates a new pixel entry in the database
    ///
    /// # Arguments
    ///
    /// * `creation` - The data required to create the pixel
    ///
    /// # Returns
    ///
    /// A result containing the created `PixelModel` or a database error
    pub async fn create_new(creation: PixelCreation) -> ModelResult<Vec<PixelResult>> {
        let mut conn = db!().begin().await?;

        let mut pixel_results = Vec::new();
        for [x, y, color] in creation.pixels {
            let pixel_result = query_as!(
                PixelResult,
                r#"
                    INSERT INTO pixels (x, y, user_id, color)
                    VALUES ($1, $2, $3, $4)
                    ON CONFLICT (x, y) DO UPDATE SET
                        user_id = $3,
                        color = $4,
                        placed_at = NOW()
                    RETURNING x, y, user_id, color
                "#,
                x as i32,
                y as i32,
                Uuid::parse_str(&creation.user_id)?,
                color as i32
            )
            .fetch_one(&mut *conn)
            .await?;

            pixel_results.push(pixel_result);
        }

        conn.commit().await?;

        Ok(pixel_results)
    }

    pub fn to_result(self) -> PixelResult {
        PixelResult {
            x: self.x,
            y: self.y,
            user_id: self.user_id.into(),
            color: self.color,
        }
    }
}
