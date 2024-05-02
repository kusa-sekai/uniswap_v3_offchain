use hex::encode;
use rand::Rng;
const MAX_INDEX: u32 = 20;
const MIN_INDEX: u32 = 0;

// INFO: index range is between 0 and 20.
// INFO: tick_spacing is 1.

#[derive(Debug)]
struct Pool {
    token_a_address: String,
    token_b_address: String,
    current_tick_index: u32,
    sqrt_price: f64,
    tick_spacing: u32,
    bitmap: [u32; 20],
    ticks: Vec<Tick>,
}

#[derive(Debug)]
struct Account {
    address: String,
    a_balance: f64,
    b_balance: f64,
}

#[derive(Debug)]
struct Position {
    account_address: String,
    upper_tick_index: u32,
    lower_tick_index: u32,
    liquidity: u32,
}

#[derive(Debug)]
struct Tick {
    index: u32,
    liquidity_gross: u32,
    liquidity_net: i32,
}

impl Position {
    fn new(
        pool: &mut Pool,
        account: &mut Account,
        upper_tick_index: u32,
        lower_tick_index: u32,
        liquidity: u32,
    ) -> Result<Self, String> {
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

        let deposit_token_a_amount =
            (liquidity as f64 / sqrt_price) - (liquidity as f64 / sqrt_price_at_upper);
        let deposit_token_b_amount =
            (liquidity as f64 * sqrt_price) - (liquidity as f64 * sqrt_price_at_lower);

        if (account.a_balance as f64) < deposit_token_a_amount
            || (account.b_balance as f64) < deposit_token_b_amount
        {
            return Err("enough balance".to_string());
        }

        account.a_balance -= deposit_token_a_amount;
        account.b_balance -= deposit_token_b_amount;

        let upper_tick_opinion = get_tick(pool, upper_tick_index);

        if let Some(tick) = upper_tick_opinion {
            tick.liquidity_gross += liquidity;
            tick.liquidity_net -= liquidity as i32;
        } else {
            pool.ticks.push(Tick {
                index: upper_tick_index,
                liquidity_gross: liquidity,
                liquidity_net: -(liquidity as i32),
            });
            pool.bitmap[upper_tick_index as usize] = 1;
        }

        let lower_tick_opinion = get_tick(pool, lower_tick_index);

        if let Some(tick) = lower_tick_opinion {
            tick.liquidity_gross += liquidity;
            tick.liquidity_net += liquidity as i32;
        } else {
            pool.ticks.push(Tick {
                index: lower_tick_index,
                liquidity_gross: liquidity,
                liquidity_net: liquidity as i32,
            });
            pool.bitmap[lower_tick_index as usize] = 1;
        }

        Ok(Position {
            account_address: account.address.clone(),
            upper_tick_index,
            lower_tick_index,
            liquidity,
        })
    }

    fn close(mut self, mut pool: Pool, mut account: Account) {
        let current_sqrt_price = pool.sqrt_price;
        let current_tick_index = pool.current_tick_index;
        let liquidity = self.liquidity;
        let mut target_sqrt_price: f64 = 0.0;

        if current_tick_index < self.lower_tick_index {
            target_sqrt_price = get_sqrt_price_at_tick(self.lower_tick_index)
        }

        if current_tick_index > self.upper_tick_index {
            target_sqrt_price = get_sqrt_price_at_tick(self.upper_tick_index)
        }
        let a_amount = liquidity as f64 / target_sqrt_price;
        let b_amount = liquidity as f64 * target_sqrt_price;

        account.a_balance += a_amount;
        account.b_balance += b_amount;

        let upper_tick_opinion = get_tick(&mut pool, self.upper_tick_index);

        if let Some(tick) = upper_tick_opinion {
            tick.liquidity_gross -= liquidity;
            tick.liquidity_net += liquidity as i32;

            if tick.liquidity_gross == 0 {
                pool.bitmap[self.upper_tick_index as usize] = 0;
            }
        }

        let lower_tick_opinion: Option<&mut Tick> = get_tick(&mut pool, self.lower_tick_index);

        if let Some(tick) = lower_tick_opinion {
            tick.liquidity_gross -= liquidity;
            tick.liquidity_net -= liquidity as i32;

            if tick.liquidity_gross == 0 {
                pool.bitmap[self.lower_tick_index as usize] = 0;
            }
        }

        drop(self);
    }
}

impl Pool {
    fn new() -> Self {

        let token_a_address = get_random_token_adddress();
        let token_b_address = get_random_token_adddress();
        let current_tick_index: u32 = 10;
        let tick_spacing: u32 = 0;
        let bitmap: [u32; 20] = [0; 20];
        let ticks = Vec::new();

        let base: f64 = 1.0001;
        let price = base.powf(current_tick_index as f64);
        let sqrt_price: f64 = price.sqrt();

        Pool {
            token_a_address,
            token_b_address,
            current_tick_index,
            tick_spacing,
            bitmap,
            ticks,
            sqrt_price,
        }
    }
}

impl Account {
    fn new(a_balance: f64, b_balance: f64) -> Self {
        let mut rng = rand::thread_rng();
        let address: [u8; 20] = rng.gen();
        let hex_address = encode(address);
        Account {
            address: hex_address,
            a_balance,
            b_balance,
        }
    }

    fn swap_a_to_b(&mut self, pool: &mut Pool, a_amount: f64) -> Result<(), String> {

        let mut liquidity = 0;

        let tick_opinion: Option<&mut Tick> = pool
            .ticks
            .iter_mut()
            .find(|tick| tick.index == pool.current_tick_index);

        if let Some(tick) = tick_opinion {
            liquidity = tick.liquidity_gross;
        }

        if liquidity == 0 {
            return Err("not enough liquidity".to_string());
        }

        let mut remaining_a_amount = a_amount;

        while remaining_a_amount > 0 as f64 {
            // INFO: Δx= L/Δ√p
            //  Δ√p = L / Δx
            let delta_sqrt_p: f64 = liquidity as f64 / remaining_a_amount;
            let target_sqrt_price: f64 = pool.sqrt_price + delta_sqrt_p;

            let mut next_tick_sqrt_price: f64 = get_sqrt_price_at_tick(pool.current_tick_index + 1);

            if target_sqrt_price <= next_tick_sqrt_price {
                // Δy= L*Δ√p
                let delta_y: f64 = liquidity as f64 * delta_sqrt_p;
                let delta_x: f64 = liquidity as f64 / delta_sqrt_p;
                self.a_balance += delta_x;
                self.b_balance += delta_y;
                pool.sqrt_price = target_sqrt_price;
            } else {
                // next_tick_sqrt_priceを完全に超えたらtickを更新する。
                if pool.bitmap[(pool.current_tick_index + 1) as usize] == 1 {

                    let mut delta_y: f64 = liquidity as f64 * (next_tick_sqrt_price - pool.sqrt_price);
                    let mut delta_x: f64 = liquidity as f64 * (1 as f64 /next_tick_sqrt_price - 1 as f64 /pool.sqrt_price);

                    remaining_a_amount -= delta_x;

                    pool.current_tick_index = pool.current_tick_index + 1;
                    pool.sqrt_price = next_tick_sqrt_price;

                    let mut curret_tick_index = pool.current_tick_index;

                    let next_tick_opinion: Option<&mut Tick> =
                        get_tick(pool, curret_tick_index);

                    if let Some(mut next_tick) = next_tick_opinion {
                        liquidity = next_tick.liquidity_gross;
                    }

                    self.a_balance += delta_x;
                    self.b_balance += delta_y;

                    if pool.bitmap[pool.current_tick_index as usize] == 0 {
                        break;
                    }
                } else {
                    let delta_y: f64 = liquidity as f64 * (next_tick_sqrt_price - pool.sqrt_price);
                    let delta_x: f64 = liquidity as f64 * (1 as f64 /next_tick_sqrt_price - 1 as f64 /pool.sqrt_price);
                    self.a_balance += delta_x;
                    self.b_balance += delta_y;
                    pool.sqrt_price = next_tick_sqrt_price;
                    break
                }
            }
        }
        Ok(())
    }
}

fn get_sqrt_price_at_tick(tick_index: u32) -> f64 {
    let base: f64 = 1.0001;
    let price = base.powf(tick_index as f64);
    price.sqrt()
}

fn get_tick(pool: &mut Pool, tick_index: u32) -> Option<&mut Tick> {
    let upper_tick_opinion: Option<&mut Tick> =
        pool.ticks.iter_mut().find(|tick| tick.index == tick_index);
    upper_tick_opinion
}

fn get_random_token_adddress() -> String {
    let mut rng = rand::thread_rng();
    let token_address: [u8; 20] = rng.gen();
    let hex_token_address = encode(token_address);
    hex_token_address
}

fn main() {
    let mut account = Account::new(1000 as f64, 1000 as f64);
    let mut pool = Pool::new();
    // Position::new(&mut pool, &mut account, 19, 10, 100000);
    Position::new(&mut pool, &mut account, 15, 10, 1000000);
    println!("{:?}", account);
    account.swap_a_to_b(&mut pool, 100 as f64);
    println!("{:?}", account);
    println!("{:?}", pool);
}
