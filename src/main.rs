use setup::SetupGames;
mod setup;


fn main() {
    println!("Hello, world!");
    let uji = SetupGames{name:"Lyra Engine".to_string()};
    uji.run()
}
