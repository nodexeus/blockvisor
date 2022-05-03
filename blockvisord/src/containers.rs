use anyhow::Result;
use async_trait::async_trait;

#[allow(dead_code)]
pub enum ContainerStatus {
    Created,
}
#[async_trait]
pub trait NodeContainer {
    /// Creates a new container with `id`.
    async fn create(id: &str) -> Result<Self>
    where
        Self: Sized;

    /// Returns the container's `id`.
    fn id(&self) -> String;

    /// Starts the container.
    async fn start(&self) -> Result<()>;

    /// Returns the state of the container.
    async fn state(&self) -> Result<()>;

    /// Kills the running container.
    async fn kill(&self) -> Result<()>;

    /// Deletes the container.
    async fn delete(&self) -> Result<()>;
}

pub struct LinuxNode {
    pub id: String,
}

#[async_trait]
impl NodeContainer for LinuxNode {
    async fn create(_id: &str) -> Result<Self> {
        unimplemented!()
    }

    fn id(&self) -> String {
        self.id.to_owned()
    }

    async fn start(&self) -> Result<()> {
        unimplemented!()
    }

    async fn state(&self) -> Result<()> {
        unimplemented!()
    }

    async fn kill(&self) -> Result<()> {
        unimplemented!()
    }

    /// Deletes the container.
    async fn delete(&self) -> Result<()> {
        unimplemented!()
    }
}
