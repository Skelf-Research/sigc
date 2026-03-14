//! Trading safety systems
//!
//! Provides circuit breakers, position limits, order validation, and kill switches.

use sig_types::{Result, SigcError};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

/// Global kill switch for emergency stops
static KILL_SWITCH: AtomicBool = AtomicBool::new(false);

/// Activate the global kill switch
pub fn activate_kill_switch() {
    KILL_SWITCH.store(true, Ordering::SeqCst);
    tracing::error!("KILL SWITCH ACTIVATED - All trading halted");
}

/// Deactivate the global kill switch
pub fn deactivate_kill_switch() {
    KILL_SWITCH.store(false, Ordering::SeqCst);
    tracing::warn!("Kill switch deactivated - Trading resumed");
}

/// Check if kill switch is active
pub fn is_kill_switch_active() -> bool {
    KILL_SWITCH.load(Ordering::SeqCst)
}

/// Circuit breaker states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    /// Normal operation
    Closed,
    /// Allowing test requests
    HalfOpen,
    /// Blocking all requests
    Open,
}

/// Circuit breaker for protecting against cascading failures
pub struct CircuitBreaker {
    name: String,
    state: Arc<RwLock<CircuitState>>,
    failure_count: AtomicU64,
    success_count: AtomicU64,
    last_failure: Arc<RwLock<Option<Instant>>>,
    config: CircuitBreakerConfig,
}

/// Circuit breaker configuration
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Number of failures before opening
    pub failure_threshold: u64,
    /// Duration to stay open before trying half-open
    pub reset_timeout: Duration,
    /// Number of successes in half-open to close
    pub success_threshold: u64,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        CircuitBreakerConfig {
            failure_threshold: 5,
            reset_timeout: Duration::from_secs(60),
            success_threshold: 3,
        }
    }
}

impl CircuitBreaker {
    pub fn new(name: &str, config: CircuitBreakerConfig) -> Self {
        CircuitBreaker {
            name: name.to_string(),
            state: Arc::new(RwLock::new(CircuitState::Closed)),
            failure_count: AtomicU64::new(0),
            success_count: AtomicU64::new(0),
            last_failure: Arc::new(RwLock::new(None)),
            config,
        }
    }

    /// Check if request is allowed
    pub fn allow_request(&self) -> bool {
        let mut state = self.state.write().unwrap();

        match *state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                // Check if we should try half-open
                let last_failure = self.last_failure.read().unwrap();
                if let Some(time) = *last_failure {
                    if time.elapsed() >= self.config.reset_timeout {
                        *state = CircuitState::HalfOpen;
                        self.success_count.store(0, Ordering::SeqCst);
                        tracing::info!("Circuit breaker {} entering half-open state", self.name);
                        return true;
                    }
                }
                false
            }
            CircuitState::HalfOpen => true,
        }
    }

    /// Record a successful operation
    pub fn record_success(&self) {
        let mut state = self.state.write().unwrap();

        match *state {
            CircuitState::HalfOpen => {
                let count = self.success_count.fetch_add(1, Ordering::SeqCst) + 1;
                if count >= self.config.success_threshold {
                    *state = CircuitState::Closed;
                    self.failure_count.store(0, Ordering::SeqCst);
                    tracing::info!("Circuit breaker {} closed after recovery", self.name);
                }
            }
            CircuitState::Closed => {
                // Reset failure count on success
                self.failure_count.store(0, Ordering::SeqCst);
            }
            _ => {}
        }
    }

    /// Record a failed operation
    pub fn record_failure(&self) {
        let mut state = self.state.write().unwrap();

        match *state {
            CircuitState::Closed => {
                let count = self.failure_count.fetch_add(1, Ordering::SeqCst) + 1;
                if count >= self.config.failure_threshold {
                    *state = CircuitState::Open;
                    *self.last_failure.write().unwrap() = Some(Instant::now());
                    tracing::warn!(
                        "Circuit breaker {} opened after {} failures",
                        self.name,
                        count
                    );
                }
            }
            CircuitState::HalfOpen => {
                *state = CircuitState::Open;
                *self.last_failure.write().unwrap() = Some(Instant::now());
                tracing::warn!("Circuit breaker {} reopened after failure in half-open", self.name);
            }
            _ => {}
        }
    }

    /// Get current state
    pub fn state(&self) -> CircuitState {
        *self.state.read().unwrap()
    }

    /// Reset the circuit breaker
    pub fn reset(&self) {
        *self.state.write().unwrap() = CircuitState::Closed;
        self.failure_count.store(0, Ordering::SeqCst);
        self.success_count.store(0, Ordering::SeqCst);
        *self.last_failure.write().unwrap() = None;
    }
}

/// Position limits configuration
#[derive(Debug, Clone)]
pub struct PositionLimits {
    /// Maximum position size per asset (absolute value)
    pub max_position_value: f64,
    /// Maximum position as fraction of portfolio
    pub max_position_pct: f64,
    /// Maximum gross exposure
    pub max_gross_exposure: f64,
    /// Maximum net exposure
    pub max_net_exposure: f64,
    /// Maximum sector exposure
    pub max_sector_exposure: f64,
    /// Maximum number of positions
    pub max_positions: usize,
}

impl Default for PositionLimits {
    fn default() -> Self {
        PositionLimits {
            max_position_value: 1_000_000.0,
            max_position_pct: 0.10,
            max_gross_exposure: 2.0,
            max_net_exposure: 1.0,
            max_sector_exposure: 0.30,
            max_positions: 100,
        }
    }
}

/// Position limit enforcer
pub struct PositionLimitEnforcer {
    limits: PositionLimits,
    current_positions: Arc<RwLock<HashMap<String, f64>>>,
    sector_map: Arc<RwLock<HashMap<String, String>>>,
}

impl PositionLimitEnforcer {
    pub fn new(limits: PositionLimits) -> Self {
        PositionLimitEnforcer {
            limits,
            current_positions: Arc::new(RwLock::new(HashMap::new())),
            sector_map: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Set sector mapping for an asset
    pub fn set_sector(&self, asset: &str, sector: &str) {
        self.sector_map
            .write()
            .unwrap()
            .insert(asset.to_string(), sector.to_string());
    }

    /// Update current position
    pub fn update_position(&self, asset: &str, value: f64) {
        self.current_positions
            .write()
            .unwrap()
            .insert(asset.to_string(), value);
    }

    /// Check if a proposed trade would violate limits
    pub fn check_trade(
        &self,
        asset: &str,
        proposed_value: f64,
        portfolio_value: f64,
    ) -> Result<()> {
        // Check kill switch first
        if is_kill_switch_active() {
            return Err(SigcError::Runtime("Kill switch is active".into()));
        }

        let positions = self.current_positions.read().unwrap();

        // Check max position value
        if proposed_value.abs() > self.limits.max_position_value {
            return Err(SigcError::Runtime(format!(
                "Position value {} exceeds limit {}",
                proposed_value, self.limits.max_position_value
            )));
        }

        // Check max position percentage
        if portfolio_value > 0.0 {
            let pct = proposed_value.abs() / portfolio_value;
            if pct > self.limits.max_position_pct {
                return Err(SigcError::Runtime(format!(
                    "Position {}% exceeds limit {}%",
                    pct * 100.0,
                    self.limits.max_position_pct * 100.0
                )));
            }
        }

        // Check number of positions
        let current_count = positions.len();
        if !positions.contains_key(asset) && current_count >= self.limits.max_positions {
            return Err(SigcError::Runtime(format!(
                "Would exceed max positions limit of {}",
                self.limits.max_positions
            )));
        }

        // Calculate new gross/net exposure
        let mut gross = 0.0;
        let mut net = 0.0;
        for (sym, val) in positions.iter() {
            if sym == asset {
                gross += proposed_value.abs();
                net += proposed_value;
            } else {
                gross += val.abs();
                net += val;
            }
        }
        if !positions.contains_key(asset) {
            gross += proposed_value.abs();
            net += proposed_value;
        }

        if portfolio_value > 0.0 {
            let gross_pct = gross / portfolio_value;
            let net_pct = net.abs() / portfolio_value;

            if gross_pct > self.limits.max_gross_exposure {
                return Err(SigcError::Runtime(format!(
                    "Gross exposure {}% exceeds limit {}%",
                    gross_pct * 100.0,
                    self.limits.max_gross_exposure * 100.0
                )));
            }

            if net_pct > self.limits.max_net_exposure {
                return Err(SigcError::Runtime(format!(
                    "Net exposure {}% exceeds limit {}%",
                    net_pct * 100.0,
                    self.limits.max_net_exposure * 100.0
                )));
            }
        }

        // Check sector exposure
        let sector_map = self.sector_map.read().unwrap();
        if let Some(sector) = sector_map.get(asset) {
            let mut sector_exposure = proposed_value.abs();
            for (sym, val) in positions.iter() {
                if sym != asset {
                    if let Some(sym_sector) = sector_map.get(sym) {
                        if sym_sector == sector {
                            sector_exposure += val.abs();
                        }
                    }
                }
            }

            if portfolio_value > 0.0 {
                let sector_pct = sector_exposure / portfolio_value;
                if sector_pct > self.limits.max_sector_exposure {
                    return Err(SigcError::Runtime(format!(
                        "Sector {} exposure {}% exceeds limit {}%",
                        sector,
                        sector_pct * 100.0,
                        self.limits.max_sector_exposure * 100.0
                    )));
                }
            }
        }

        Ok(())
    }

    /// Get current gross exposure
    pub fn gross_exposure(&self) -> f64 {
        self.current_positions
            .read()
            .unwrap()
            .values()
            .map(|v| v.abs())
            .sum()
    }

    /// Get current net exposure
    pub fn net_exposure(&self) -> f64 {
        self.current_positions.read().unwrap().values().sum()
    }
}

/// Order validation rules
#[derive(Debug, Clone)]
pub struct OrderValidationRules {
    /// Maximum order value
    pub max_order_value: f64,
    /// Maximum order as percentage of ADV
    pub max_adv_pct: f64,
    /// Minimum order value
    pub min_order_value: f64,
    /// Price deviation limit from reference
    pub max_price_deviation: f64,
    /// Blocked symbols
    pub blocked_symbols: Vec<String>,
}

impl Default for OrderValidationRules {
    fn default() -> Self {
        OrderValidationRules {
            max_order_value: 500_000.0,
            max_adv_pct: 0.10,
            min_order_value: 100.0,
            max_price_deviation: 0.05,
            blocked_symbols: Vec::new(),
        }
    }
}

/// Order to be validated
#[derive(Debug, Clone)]
pub struct Order {
    pub symbol: String,
    pub side: OrderSide,
    pub quantity: f64,
    pub price: Option<f64>,
    pub order_type: OrderType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderType {
    Market,
    Limit,
    Stop,
    StopLimit,
}

/// Order validator
pub struct OrderValidator {
    rules: OrderValidationRules,
    adv_data: Arc<RwLock<HashMap<String, f64>>>,
    reference_prices: Arc<RwLock<HashMap<String, f64>>>,
}

impl OrderValidator {
    pub fn new(rules: OrderValidationRules) -> Self {
        OrderValidator {
            rules,
            adv_data: Arc::new(RwLock::new(HashMap::new())),
            reference_prices: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Set ADV (average daily volume) for a symbol
    pub fn set_adv(&self, symbol: &str, adv: f64) {
        self.adv_data
            .write()
            .unwrap()
            .insert(symbol.to_string(), adv);
    }

    /// Set reference price for a symbol
    pub fn set_reference_price(&self, symbol: &str, price: f64) {
        self.reference_prices
            .write()
            .unwrap()
            .insert(symbol.to_string(), price);
    }

    /// Validate an order
    pub fn validate(&self, order: &Order) -> Result<()> {
        // Check kill switch
        if is_kill_switch_active() {
            return Err(SigcError::Runtime("Kill switch is active".into()));
        }

        // Check blocked symbols
        if self.rules.blocked_symbols.contains(&order.symbol) {
            return Err(SigcError::Runtime(format!(
                "Symbol {} is blocked",
                order.symbol
            )));
        }

        // Calculate order value
        let price = order.price.unwrap_or_else(|| {
            self.reference_prices
                .read()
                .unwrap()
                .get(&order.symbol)
                .copied()
                .unwrap_or(0.0)
        });

        let order_value = order.quantity * price;

        // Check order value limits
        if order_value > self.rules.max_order_value {
            return Err(SigcError::Runtime(format!(
                "Order value {} exceeds limit {}",
                order_value, self.rules.max_order_value
            )));
        }

        if order_value < self.rules.min_order_value {
            return Err(SigcError::Runtime(format!(
                "Order value {} below minimum {}",
                order_value, self.rules.min_order_value
            )));
        }

        // Check ADV limit
        let adv_data = self.adv_data.read().unwrap();
        if let Some(&adv) = adv_data.get(&order.symbol) {
            if adv > 0.0 {
                let adv_pct = order_value / adv;
                if adv_pct > self.rules.max_adv_pct {
                    return Err(SigcError::Runtime(format!(
                        "Order is {}% of ADV, exceeds limit {}%",
                        adv_pct * 100.0,
                        self.rules.max_adv_pct * 100.0
                    )));
                }
            }
        }

        // Check price deviation for limit orders
        if let (Some(order_price), OrderType::Limit | OrderType::StopLimit) =
            (order.price, order.order_type)
        {
            let ref_prices = self.reference_prices.read().unwrap();
            if let Some(&ref_price) = ref_prices.get(&order.symbol) {
                if ref_price > 0.0 {
                    let deviation = (order_price - ref_price).abs() / ref_price;
                    if deviation > self.rules.max_price_deviation {
                        return Err(SigcError::Runtime(format!(
                            "Price deviation {}% exceeds limit {}%",
                            deviation * 100.0,
                            self.rules.max_price_deviation * 100.0
                        )));
                    }
                }
            }
        }

        Ok(())
    }
}

/// Rate limiter using token bucket algorithm
pub struct RateLimiter {
    name: String,
    capacity: u64,
    tokens: AtomicU64,
    refill_rate: u64,
    last_refill: Arc<RwLock<Instant>>,
}

impl RateLimiter {
    /// Create a new rate limiter
    pub fn new(name: &str, capacity: u64, refill_rate: u64) -> Self {
        RateLimiter {
            name: name.to_string(),
            capacity,
            tokens: AtomicU64::new(capacity),
            refill_rate,
            last_refill: Arc::new(RwLock::new(Instant::now())),
        }
    }

    /// Try to acquire a token
    pub fn try_acquire(&self) -> bool {
        self.refill();

        loop {
            let current = self.tokens.load(Ordering::SeqCst);
            if current == 0 {
                return false;
            }
            if self
                .tokens
                .compare_exchange(current, current - 1, Ordering::SeqCst, Ordering::SeqCst)
                .is_ok()
            {
                return true;
            }
        }
    }

    /// Acquire a token, blocking if necessary
    pub fn acquire(&self) {
        while !self.try_acquire() {
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    /// Refill tokens based on elapsed time
    fn refill(&self) {
        let mut last = self.last_refill.write().unwrap();
        let elapsed = last.elapsed();
        let new_tokens = (elapsed.as_millis() as u64 * self.refill_rate) / 1000;

        if new_tokens > 0 {
            let current = self.tokens.load(Ordering::SeqCst);
            let new_total = (current + new_tokens).min(self.capacity);
            self.tokens.store(new_total, Ordering::SeqCst);
            *last = Instant::now();
        }
    }

    /// Get current token count
    pub fn available(&self) -> u64 {
        self.refill();
        self.tokens.load(Ordering::SeqCst)
    }
}

/// Safety manager combining all safety systems
pub struct SafetyManager {
    pub circuit_breakers: HashMap<String, CircuitBreaker>,
    pub position_enforcer: PositionLimitEnforcer,
    pub order_validator: OrderValidator,
    pub rate_limiters: HashMap<String, RateLimiter>,
}

impl SafetyManager {
    pub fn new(
        position_limits: PositionLimits,
        order_rules: OrderValidationRules,
    ) -> Self {
        SafetyManager {
            circuit_breakers: HashMap::new(),
            position_enforcer: PositionLimitEnforcer::new(position_limits),
            order_validator: OrderValidator::new(order_rules),
            rate_limiters: HashMap::new(),
        }
    }

    /// Add a circuit breaker
    pub fn add_circuit_breaker(&mut self, name: &str, config: CircuitBreakerConfig) {
        self.circuit_breakers
            .insert(name.to_string(), CircuitBreaker::new(name, config));
    }

    /// Add a rate limiter
    pub fn add_rate_limiter(&mut self, name: &str, capacity: u64, refill_rate: u64) {
        self.rate_limiters
            .insert(name.to_string(), RateLimiter::new(name, capacity, refill_rate));
    }

    /// Check all safety systems before executing
    pub fn pre_trade_check(&self, order: &Order, portfolio_value: f64) -> Result<()> {
        // Kill switch
        if is_kill_switch_active() {
            return Err(SigcError::Runtime("Kill switch is active".into()));
        }

        // Circuit breakers
        for (name, cb) in &self.circuit_breakers {
            if !cb.allow_request() {
                return Err(SigcError::Runtime(format!(
                    "Circuit breaker {} is open",
                    name
                )));
            }
        }

        // Rate limiting
        for (name, rl) in &self.rate_limiters {
            if !rl.try_acquire() {
                return Err(SigcError::Runtime(format!(
                    "Rate limit {} exceeded",
                    name
                )));
            }
        }

        // Order validation
        self.order_validator.validate(order)?;

        // Position limits
        let order_value = order.quantity
            * order.price.unwrap_or_else(|| {
                self.order_validator
                    .reference_prices
                    .read()
                    .unwrap()
                    .get(&order.symbol)
                    .copied()
                    .unwrap_or(0.0)
            });

        let signed_value = match order.side {
            OrderSide::Buy => order_value,
            OrderSide::Sell => -order_value,
        };

        self.position_enforcer
            .check_trade(&order.symbol, signed_value, portfolio_value)?;

        Ok(())
    }

    /// Get overall system status
    pub fn system_status(&self) -> SafetyStatus {
        let kill_switch = is_kill_switch_active();

        let mut circuit_breaker_status = HashMap::new();
        for (name, cb) in &self.circuit_breakers {
            circuit_breaker_status.insert(name.clone(), cb.state());
        }

        SafetyStatus {
            kill_switch_active: kill_switch,
            circuit_breaker_states: circuit_breaker_status,
            gross_exposure: self.position_enforcer.gross_exposure(),
            net_exposure: self.position_enforcer.net_exposure(),
        }
    }
}

/// Overall safety system status
#[derive(Debug, Clone)]
pub struct SafetyStatus {
    pub kill_switch_active: bool,
    pub circuit_breaker_states: HashMap<String, CircuitState>,
    pub gross_exposure: f64,
    pub net_exposure: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kill_switch() {
        deactivate_kill_switch();
        assert!(!is_kill_switch_active());

        activate_kill_switch();
        assert!(is_kill_switch_active());

        deactivate_kill_switch();
        assert!(!is_kill_switch_active());
    }

    #[test]
    fn test_circuit_breaker_closed() {
        let cb = CircuitBreaker::new("test", CircuitBreakerConfig::default());
        assert_eq!(cb.state(), CircuitState::Closed);
        assert!(cb.allow_request());
    }

    #[test]
    fn test_circuit_breaker_opens() {
        let config = CircuitBreakerConfig {
            failure_threshold: 3,
            reset_timeout: Duration::from_secs(60),
            success_threshold: 2,
        };
        let cb = CircuitBreaker::new("test", config);

        // Record failures
        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Closed);

        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);
        assert!(!cb.allow_request());
    }

    #[test]
    fn test_circuit_breaker_recovery() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            reset_timeout: Duration::from_millis(10),
            success_threshold: 2,
        };
        let cb = CircuitBreaker::new("test", config);

        // Open the breaker
        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);

        // Wait for reset timeout
        std::thread::sleep(Duration::from_millis(20));

        // Should transition to half-open
        assert!(cb.allow_request());
        assert_eq!(cb.state(), CircuitState::HalfOpen);

        // Record successes to close
        cb.record_success();
        cb.record_success();
        assert_eq!(cb.state(), CircuitState::Closed);
    }

    #[test]
    fn test_position_limits_max_value() {
        let limits = PositionLimits {
            max_position_value: 100_000.0,
            ..Default::default()
        };
        let enforcer = PositionLimitEnforcer::new(limits);

        // Should pass
        assert!(enforcer.check_trade("AAPL", 50_000.0, 1_000_000.0).is_ok());

        // Should fail
        assert!(enforcer.check_trade("AAPL", 150_000.0, 1_000_000.0).is_err());
    }

    #[test]
    fn test_position_limits_max_pct() {
        let limits = PositionLimits {
            max_position_pct: 0.05, // 5%
            ..Default::default()
        };
        let enforcer = PositionLimitEnforcer::new(limits);

        // Should pass (3%)
        assert!(enforcer.check_trade("AAPL", 30_000.0, 1_000_000.0).is_ok());

        // Should fail (10%)
        assert!(enforcer.check_trade("AAPL", 100_000.0, 1_000_000.0).is_err());
    }

    #[test]
    fn test_order_validation() {
        let rules = OrderValidationRules {
            max_order_value: 100_000.0,
            min_order_value: 100.0,
            ..Default::default()
        };
        let validator = OrderValidator::new(rules);
        validator.set_reference_price("AAPL", 150.0);

        // Valid order
        let order = Order {
            symbol: "AAPL".to_string(),
            side: OrderSide::Buy,
            quantity: 100.0,
            price: Some(150.0),
            order_type: OrderType::Limit,
        };
        assert!(validator.validate(&order).is_ok());

        // Order too large
        let large_order = Order {
            symbol: "AAPL".to_string(),
            side: OrderSide::Buy,
            quantity: 1000.0,
            price: Some(150.0),
            order_type: OrderType::Limit,
        };
        assert!(validator.validate(&large_order).is_err());
    }

    #[test]
    fn test_blocked_symbols() {
        let rules = OrderValidationRules {
            blocked_symbols: vec!["BLOCKED".to_string()],
            ..Default::default()
        };
        let validator = OrderValidator::new(rules);
        validator.set_reference_price("BLOCKED", 100.0);

        let order = Order {
            symbol: "BLOCKED".to_string(),
            side: OrderSide::Buy,
            quantity: 10.0,
            price: Some(100.0),
            order_type: OrderType::Market,
        };
        assert!(validator.validate(&order).is_err());
    }

    #[test]
    fn test_rate_limiter() {
        let rl = RateLimiter::new("test", 3, 100);

        // Should acquire 3 tokens
        assert!(rl.try_acquire());
        assert!(rl.try_acquire());
        assert!(rl.try_acquire());

        // Should fail - no tokens
        assert!(!rl.try_acquire());

        // Wait for refill
        std::thread::sleep(Duration::from_millis(50));

        // Should have some tokens now
        assert!(rl.try_acquire());
    }

    #[test]
    fn test_safety_manager() {
        let manager = SafetyManager::new(
            PositionLimits::default(),
            OrderValidationRules::default(),
        );

        manager.order_validator.set_reference_price("AAPL", 150.0);

        let order = Order {
            symbol: "AAPL".to_string(),
            side: OrderSide::Buy,
            quantity: 100.0,
            price: Some(150.0),
            order_type: OrderType::Limit,
        };

        deactivate_kill_switch();
        assert!(manager.pre_trade_check(&order, 1_000_000.0).is_ok());

        // Activate kill switch
        activate_kill_switch();
        assert!(manager.pre_trade_check(&order, 1_000_000.0).is_err());
        deactivate_kill_switch();
    }

    #[test]
    fn test_safety_status() {
        let mut manager = SafetyManager::new(
            PositionLimits::default(),
            OrderValidationRules::default(),
        );

        manager.add_circuit_breaker("api", CircuitBreakerConfig::default());

        let status = manager.system_status();
        assert!(!status.kill_switch_active);
        assert_eq!(
            status.circuit_breaker_states.get("api"),
            Some(&CircuitState::Closed)
        );
    }
}
