use coinbase_pro_rs::{Private, Sync, SANDBOX_URL};

static KEY: &str = "1d0dc0f7b4e808d430b95d8fed7df3ea";
static SECRET: &str =
    "dTUic8DZPqkS77vxhJFEX5IBr13FcFHTzWYOARgT9kDWGdN03uvxBbH/hVy8f4O5RDmuf+9wNpEfhYhw2FCWyA==";
static PASSPHRASE: &str = "sandbox";

fn main() {
    let client: Private<Sync> = Private::new(SANDBOX_URL, KEY, SECRET, PASSPHRASE);
    let accounts = client.get_accounts().unwrap();

    let btc = accounts.iter().find(|x| x.currency == "BTC").unwrap();
    println!("{}.  balance: {:?}", btc.currency, btc.balance);
    println!("{}.available: {:?}", btc.currency, btc.available);
    println!("{}.     hold: {:?}", btc.currency, btc.hold);
}
