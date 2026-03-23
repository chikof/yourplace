use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, query_as};
use uuid::Uuid;

use crate::CoinTransactionResult;
use crate::db;
use crate::utils::error::ModelResult;

/// Represents a pixel entry in the database
#[derive(FromRow)]
pub struct PixelModel {
    /// ID of the community/canvas the pixel belongs to
    community_id: Option<Uuid>,

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
    pub pixels: Vec<[i64; 3]>,

    /// ID of the user who is creating the pixel entry (as a string)
    pub user_id: String,

    /// Optional community/canvas identifier for scoping the draw action
    pub community_id: Option<String>,
}

/// Represents the result of a pixel operation
#[derive(Serialize, Clone)]
pub struct PixelResult {
    /// The community/canvas identifier when present
    pub community_id: Option<String>,

    /// X coordinate of the pixel
    pub x: i64,

    /// Y coordinate of the pixel
    pub y: i64,

    /// uuid of the user who placed the pixel (as a string)
    pub user_id: String,

    /// Color of the pixel as a 32-bit integer (hexadecimal representation)
    pub color: i32,
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
        let community_id =
            creation.community_id.as_ref().map(|id| Uuid::parse_str(id)).transpose()?;
        let user_id = Uuid::parse_str(&creation.user_id)?;
        let reward = creation.pixels.len() as i32;

        for [x, y, color] in creation.pixels {
            let pixel_result = query_as!(
                PixelResult,
                r#"
                    INSERT INTO pixels (community_id, x, y, user_id, color)
                    VALUES ($1, $2, $3, $4, $5)
                    ON CONFLICT (community_id, x, y) DO UPDATE SET
                        user_id = $4,
                        color = $5,
                        placed_at = NOW()
                    RETURNING community_id::text, x, y, user_id, color
                "#,
                community_id,
                x as i32,
                y as i32,
                user_id,
                color as i32
            )
            .fetch_one(&mut *conn)
            .await?;

            pixel_results.push(pixel_result);
        }

        if reward > 0 {
            let reference = creation
                .community_id
                .clone()
                .map(|id| format!("Placed {} pixels in community {}", reward, id));
            let _ =
                CoinTransactionResult::reward_pixels_in_tx(&mut conn, &user_id, reward, reference)
                    .await?;
        }

        conn.commit().await?;

        Ok(pixel_results)
    }

    pub async fn list_for_community(community_id: &str) -> ModelResult<Vec<PixelResult>> {
        let pixels = query_as!(
            PixelResult,
            r#"
                SELECT community_id::text, x, y, user_id, color
                FROM pixels
                WHERE community_id = $1
            "#,
            Uuid::parse_str(community_id)?
        )
        .fetch_all(db!())
        .await?;

        Ok(pixels)
    }

    pub fn to_result(self) -> PixelResult {
        PixelResult {
            community_id: self.community_id.map(|id| id.into()),
            x: self.x,
            y: self.y,
            user_id: self.user_id.into(),
            color: self.color,
        }
    }

    /// Compresses a list of pixels into run-length encoded sequences ordered by row
    /// to reduce bandwidth during canvas fetches.
    pub fn compress(pixels: &[PixelResult]) -> Vec<CompressedPixel> {
        let mut sorted = pixels.to_vec();
        sorted.sort_by_key(|p| (p.y, p.x));

        let mut compressed = Vec::new();
        let mut iter = sorted.into_iter().peekable();

        while let Some(pixel) = iter.next() {
            let mut length = 1;
            while let Some(next) = iter.peek() {
                if next.y == pixel.y && next.x == pixel.x + length && next.color == pixel.color {
                    length += 1;
                    iter.next();
                } else {
                    break;
                }
            }

            compressed.push(CompressedPixel {
                community_id: pixel.community_id.clone(),
                x: pixel.x,
                y: pixel.y,
                run: length,
                color: pixel.color,
            });
        }

        compressed
    }
}

/// Run-length encoded pixel sequence used for efficient payloads.
#[derive(Serialize)]
pub struct CompressedPixel {
    pub community_id: Option<String>,
    pub x: i64,
    pub y: i64,
    pub run: i64,
    pub color: i32,
}
