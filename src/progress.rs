use async_trait::async_trait;

#[async_trait]
pub trait Progress {
    /// Sets the file size with `size`
    async fn set_file_size(&mut self, size: usize);

    /// Add to the progress with `amount`
    async fn add_to_progress(&mut self, amount: usize);

    /// In the case of corrupted bytes we want to reduce the progress, or reset it to 0.
    async fn remove_from_progress(&mut self, bytes: usize);
}