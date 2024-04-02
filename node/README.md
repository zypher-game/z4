# Z4 node

Universal engine node when game is running in vm

## template
```
#[storage]
struct Shoot {
    players: HashMap<PlayerId, Player>
}

#[online]
fn online(shoot: &mut Shoot, player: PlayerId) -> Result<HandleResult> {
    //
}

#[offline]
fn offline(shoot: &mut Shoot, player: PlayerId) -> Result<HandleResult> {
    //
}

#[method]
fn shoot(shoot: &mut Shoot, player_a: PlayerId, player_b: PlayerId) -> Result<HandleResult> {
    //
}

#[method]
fn regain(shoot: &mut Shoot, player: PlayerId) -> Result<HandleResult> {
    //
}

#[task]
fn timer1(shoot: &mut Shoot) -> Result<(u32, HandleResult)> {
    //
}

#[task]
fn timer2(shoot: &mut Shoot) -> Result<(u32, HandleResult)> {
    //
}
```
