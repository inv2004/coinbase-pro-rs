API:
  private
  public
  async
  sync

cl = new_auth(login, pass) -> struct CB()
impl CB() {
    get_wallets() -> ok // private
    get_time() -> ok // public
}

cl = new_auth(login, pass) -> struct CB<auth>
impl CB<auth> {
    get_wallets() -> ok // private
}

cl = new() -> struct CB<()>
impl CB<()> {
    get_time() -> ok // public
}