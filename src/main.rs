use rand::Rng;
use hex::encode;
const MAX_INDEX: u32 = 20;
const MIN_INDEX: u32 = 0;

// INFO: index range is between 0 and 20.
// INFO: tick_spacing is 1.
struct Pool {
    token_a_address: String,
    token_b_address: String,
    current_tick_index: u32,
    sqrt_price: f64,
    tick_spacing: u32,
    bitmap: [u32; 20],
    ticks: Vec<Tick>,
}

struct Account {
    address: String,
    a_balance: f64,
    b_balance: f64
}

struct Position {
    account_address: String,
    upper_tick_index: u32,
    lower_tick_index: u32,
    liquidity: u32,
}

struct Tick {
    index: u32,
    liquidity_gross: u32,
    liquidity_net: i32,
}

impl Position {
    fn new(mut pool: Pool, mut account: Account, upper_tick_index: u32, lower_tick_index: u32, liquidity: u32) -> Result<Self, String> {

        if upper_tick_index > MAX_INDEX {
            return Err("upper_tick_index must be 20 or less".to_string());
        }
        if lower_tick_index < MIN_INDEX {
            return Err("lower_tick_index must be 0 or more".to_string());
        }

        let sqrt_price: f64 = pool.sqrt_price;
        let base: f64 = 1.0001;
        let price_at_lower = base.powf(lower_tick_index as f64);
        let sqrt_price_at_lower: f64 = price_at_lower.sqrt();
        let price_at_upper = base.powf(upper_tick_index as f64);
        let sqrt_price_at_upper: f64 = price_at_upper.sqrt();

        let deposit_token_a_amount = (liquidity as f64 / sqrt_price) - (liquidity as f64 / sqrt_price_at_upper);
        let deposit_token_b_amount = (liquidity as f64 / sqrt_price) - (liquidity as f64 / sqrt_price_at_lower);

        if (account.a_balance as f64) < deposit_token_a_amount || (account.b_balance as f64) < deposit_token_b_amount  {
            return Err("enough balance".to_string());
        }

        account.a_balance -= deposit_token_a_amount;
        account.b_balance -= deposit_token_b_amount;

        let upper_tick_opinion: Option<&mut Tick> = pool.ticks.iter_mut().find(|tick| tick.index == upper_tick_index);

        if let Some(tick) = upper_tick_opinion {
            tick.liquidity_gross += liquidity;
            tick.liquidity_net -= liquidity as i32;
        } else {
            pool.ticks.push(
            Tick {
                index: upper_tick_index,
                liquidity_gross: liquidity,
                liquidity_net: -(liquidity as i32),
            });
            pool.bitmap[upper_tick_index as usize] = 1;
        }

        let lower_tick_opinion: Option<&mut Tick> = pool.ticks.iter_mut().find(|tick| tick.index == lower_tick_index);

        if let Some(tick) = lower_tick_opinion {
            tick.liquidity_gross += liquidity;
            tick.liquidity_net += liquidity as i32;
        } else {
            pool.ticks.push(
            Tick {
                index: upper_tick_index,
                liquidity_gross: liquidity,
                liquidity_net: liquidity as i32,
            });
            pool.bitmap[lower_tick_index as usize] = 1;
        }

        Ok(Position {account_address: account.address, upper_tick_index, lower_tick_index, liquidity})
    }

    fn close(mut self) {
    }
}


impl Pool {
    fn new(token_a_address: String, token_b_address: String) -> Self {
        let current_tick_index: u32 = 10;
        let tick_spacing: u32 = 0;
        let bitmap: [u32; 20] = [0; 20];
        let ticks = Vec::new();

        let base: f64 = 1.0001;
        let price = base.powf(current_tick_index as f64);
        let sqrt_price: f64 = price.sqrt();

        Pool {token_a_address, token_b_address, current_tick_index, tick_spacing, bitmap, ticks, sqrt_price}
    }
}

impl Account {
    fn new(a_balance: f64, b_balance: f64) -> Self {
        let mut rng = rand::thread_rng();
        let address: [u8; 20] = rng.gen();
        let hex_address = encode(address);
        Account {address: hex_address, a_balance, b_balance}
    }

    // TODO: swap
}

fn main() {
    let mut account = Account::new(1000 as f64, 1000 as f64);
    let mut rng = rand::thread_rng();
    let token_a_address: [u8; 20] = rng.gen();
    let hex_token_a_address = encode(token_a_address);
    let token_b_address: [u8; 20] = rng.gen();
    let hex_token_b_address = encode(token_b_address);
    let mut pool = Pool::new(hex_token_a_address, hex_token_b_address);

}