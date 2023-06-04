use anyhow::Result;
use crate::ddd::component::{Repository, AggregateRoot};

pub trait TransactionableRepository<'conn, AR: AggregateRoot>: Repository<AR> {
    fn begin(&'conn mut self) -> Result<()>;
    fn commit(&mut self) -> Result<()>;
}

/// RepositoryComponent returns Repository.
/// This is CakePattern.
/// SEE: http://eed3si9n.com/ja/real-world-scala-dependency-injection-di/
pub trait TransactionableRepositoryComponent<'conn, AR: AggregateRoot> {
    type TransactionableRepository: TransactionableRepository<'conn, AR>;

    /// repository returns Repository.
    fn transactionable_repository(&mut self) -> &mut Self::TransactionableRepository;
}
