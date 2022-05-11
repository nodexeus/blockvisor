use anyhow::Result;
use async_trait::async_trait;

#[derive(Clone, Copy, Debug)]
pub enum ContainerStatus {
    Created,
    Started,
    Killed,
}

#[async_trait]
pub trait NodeContainer {
    /// Creates a new container with `id`.
    async fn create(id: &str) -> Result<Self>
    where
        Self: Sized;

    /// Returns the container's `id`.
    fn id(&self) -> &str;

    /// Starts the container.
    async fn start(&mut self) -> Result<()>;

    /// Returns the state of the container.
    async fn state(&self) -> Result<ContainerStatus>;

    /// Kills the running container.
    async fn kill(&mut self) -> Result<()>;

    /// Deletes the container.
    async fn delete(&mut self) -> Result<()>;
}

pub struct LinuxNode {
    id: String,
}

#[async_trait]
impl NodeContainer for LinuxNode {
    async fn create(_id: &str) -> Result<Self> {
        unimplemented!()
    }

    fn id(&self) -> &str {
        &self.id
    }

    async fn start(&mut self) -> Result<()> {
        unimplemented!()
    }

    async fn state(&self) -> Result<ContainerStatus> {
        unimplemented!()
    }

    async fn kill(&mut self) -> Result<()> {
        unimplemented!()
    }

    async fn delete(&mut self) -> Result<()> {
        unimplemented!()
    }
}

pub struct DummyNode {
    pub id: String,
    pub state: ContainerStatus,
}

#[async_trait]
impl NodeContainer for DummyNode {
    async fn create(id: &str) -> Result<Self> {
        println!("Creating node: {}", id);
        Ok(Self {
            id: id.to_owned(),
            state: ContainerStatus::Created,
        })
    }

    fn id(&self) -> &str {
        &self.id
    }

    async fn start(&mut self) -> Result<()> {
        println!("Starting node: {}", self.id());
        self.state = ContainerStatus::Started;
        Ok(())
    }

    async fn state(&self) -> Result<ContainerStatus> {
        Ok(self.state)
    }

    async fn kill(&mut self) -> Result<()> {
        println!("Killing node: {}", self.id());
        self.state = ContainerStatus::Killed;
        Ok(())
    }

    async fn delete(&mut self) -> Result<()> {
        println!("Deleting node: {}", self.id());
        self.kill().await?;
        Ok(())
    }
}
