use rand::Rng;
use hex::encode;
const MAX_INDEX: i32 = 9;
const MIN_INDEX: i32 = -9;

// INFO: index range is between -9 and 9.
// INFO: tick_spacing is 1.
struct Pool {
    token_a_address: String,
    token_b_address: String,
    current_tick_index: i32,
    tick_spacing: u32,
    bitmap: [u8; 20],
    ticks: Vec<Tick>,
}

struct Account {
    address: String,
    a_balance: i32,
    b_balance: i32
}

struct Position {
    account_address: String,
    upper_tick_index: i32,
    lower_tick_index: i32,
    liquidity: u32,
}

struct Tick {
    index: i32,
    liquidity_gross: u32,
    liquidity_net: i32,
}

impl Position {
    fn new(mut pool: Pool, account_address: String, upper_tick_index: i32, lower_tick_index: i32, liquidity: u32) -> Result<Self, String> {

        if upper_tick_index > MAX_INDEX {
            return Err("upper_tick_index must be 9 or less".to_string());
        }
        if lower_tick_index < MIN_INDEX {
            return Err("lower_tick_index must be -9 or more".to_string());
        }

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

            // TODO: bitmapの更新
        }

        Ok(Position {account_address, upper_tick_index, lower_tick_index, liquidity})
    }
}


impl Pool {
    fn new(token_a_address: String, token_b_address: String) -> Self {
        let current_tick_index: i32 = 0;
        let tick_spacing: u32 = 0;
        let bitmap: [u8; 20] = [0; 20];
        let ticks = Vec::new();
        Pool {token_a_address, token_b_address, current_tick_index, tick_spacing, bitmap, ticks}
    }
}

impl Account {
    fn new(a_balance: i32, b_balance: i32) -> Self {
        let mut rng = rand::thread_rng();
        let address: [u8; 20] = rng.gen();
        let hex_address = encode(address);
        Account {address: hex_address, a_balance, b_balance}
    }
}

fn main() {
    let mut account = Account::new(1000, 1000);
    let mut rng = rand::thread_rng();
    let token_a_address: [u8; 20] = rng.gen();
    let hex_token_a_address = encode(token_a_address);
    let token_b_address: [u8; 20] = rng.gen();
    let hex_token_b_address = encode(token_b_address);
    let mut pool = Pool::new(hex_token_a_address, hex_token_b_address);

}