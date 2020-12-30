use async_trait::async_trait;

#[async_trait]
pub trait Progress {
    async fn set_file_size(&mut self, size: usize);
    async fn get_file_size(&self) -> usize;

    /// Add to the progress with `amount`
    async fn add_to_progress(&mut self, amount: usize);

    /// In the case of corrupted bytes we want to reduce the progress, or reset it to 0.
    async fn remove_from_progress(&mut self, bytes: usize);
    async fn get_progess(&self) -> usize;
}