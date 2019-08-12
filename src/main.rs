mod init;
mod vk;

use std::collections::HashMap;


fn main()
{
    let mut map: HashMap<&str, init::Value> = HashMap::new();
    init::initialize(&mut map);
    vk::search(&map);
}
