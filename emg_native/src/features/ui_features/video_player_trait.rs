/*
 * @Author: Rais
 * @Date: 2023-04-13 13:32:25
 * @LastEditTime: 2023-04-13 15:52:29
 * @LastEditors: Rais
 * @Description:
 */
pub trait VideoPlayerT {
    type Error: std::error::Error;
    type ImageOut;
    fn new(uri: &str, live: bool) -> Result<Self, Self::Error>
    where
        Self: Sized;

    fn frame_image(&self) -> Self::ImageOut;
}
