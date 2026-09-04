use std::sync::Arc;

use crate::investment::domain::asset::{Asset, AssetClass};
use crate::investment::domain::repository::AssetRepository;
use crate::provider::id::IdGenerator;
use crate::shared::errors::InvestmentError;
use crate::shared::ids::AssetID;
use crate::shared::money::Currency;

/// Command to register a new asset in the shared catalog.
pub struct RegisterAssetCommand {
    /// Ticker symbol (e.g. `"PETR4"`, `"AAPL"`, `"BTC"`).
    pub ticker: String,
    /// Human-readable name (e.g. `"Petrobras"`, `"Apple Inc."`).
    pub name: String,
    /// Asset classification.
    pub class: AssetClass,
    /// Currency in which the asset is priced.
    pub currency: Currency,
}

/// Handler that registers a new [`Asset`] in the shared catalog.
///
/// Validates that the ticker is unique before persisting.
///
/// # Errors
///
/// - [`InvestmentError::InvariantViolation`] if the ticker or name is empty.
/// - [`InvestmentError::Repository`] if the ticker already exists or
///   persistence fails.
pub struct RegisterAssetHandler<A: AssetRepository, I: IdGenerator> {
    asset_repository: Arc<A>,
    id_generator: Arc<I>,
}

impl<A: AssetRepository, I: IdGenerator> RegisterAssetHandler<A, I> {
    /// Creates a new handler with the given dependencies.
    pub fn new(asset_repository: Arc<A>, id_generator: Arc<I>) -> Self {
        Self {
            asset_repository,
            id_generator,
        }
    }

    /// Executes the register-asset use case.
    pub async fn handle(&self, cmd: RegisterAssetCommand) -> Result<AssetID, InvestmentError> {
        // Check for duplicate ticker
        if self
            .asset_repository
            .find_by_ticker(&cmd.ticker)
            .await?
            .is_some()
        {
            return Err(InvestmentError::InvariantViolation(format!(
                "asset with ticker '{}' already exists",
                cmd.ticker,
            )));
        }

        let id = AssetID::from_uuid(self.id_generator.new_id());
        let asset = Asset::new(id, cmd.ticker, cmd.name, cmd.class, cmd.currency)?;

        self.asset_repository.save(&asset).await?;

        Ok(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::id::MockIdGenerator;
    use crate::shared::mock::MockAssetRepository;

    fn setup() -> (Arc<MockAssetRepository>, Arc<MockIdGenerator>) {
        (
            Arc::new(MockAssetRepository::new()),
            Arc::new(MockIdGenerator::new(uuid::Uuid::new_v4())),
        )
    }

    #[tokio::test]
    async fn test_register_asset_success() {
        let (repo, id_gen) = setup();
        let handler = RegisterAssetHandler::new(repo.clone(), id_gen);

        let cmd = RegisterAssetCommand {
            ticker: "PETR4".into(),
            name: "Petrobras".into(),
            class: AssetClass::Stock,
            currency: Currency::BRL,
        };

        let asset_id = handler.handle(cmd).await.unwrap();
        let asset = repo.find_by_id(asset_id).await.unwrap();
        assert!(asset.is_some());
        assert_eq!(asset.unwrap().ticker, "PETR4");
    }

    #[tokio::test]
    async fn test_register_asset_duplicate_ticker_fails() {
        let (repo, id_gen) = setup();
        let handler = RegisterAssetHandler::new(repo.clone(), id_gen);

        let cmd1 = RegisterAssetCommand {
            ticker: "PETR4".into(),
            name: "Petrobras".into(),
            class: AssetClass::Stock,
            currency: Currency::BRL,
        };
        handler.handle(cmd1).await.unwrap();

        let cmd2 = RegisterAssetCommand {
            ticker: "PETR4".into(),
            name: "Petrobras ON".into(),
            class: AssetClass::Stock,
            currency: Currency::BRL,
        };

        let result = handler.handle(cmd2).await;
        assert!(result.is_err());
    }
}
