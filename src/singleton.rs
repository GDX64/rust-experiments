use std::{
    borrow::BorrowMut,
    sync::{Mutex, Once},
};

static only_once: Once = Once::new();
static mut singleton: Option<Mutex<Singleton>> = None;

struct Singleton {
    value: i32,
}

fn lazy() -> &'static Mutex<Singleton> {
    unsafe {
        only_once.call_once(|| {
            //expensive stuff
            singleton = Some(Mutex::new(Singleton { value: 10 }));
        });
        return singleton.as_mut().unwrap();
    }
}

fn mine(func: impl FnMut() -> i32) -> impl FnMut() -> i32 {
    func
}

fn use_mine() -> impl FnMut() -> i32 {
    let mut hello = "hello".to_string();
    mine(move || {
        *(&mut hello) = "hi".to_string();
        hello.len() as i32
    })
}

fn main() {
    let s = lazy();
    let r = s.lock().unwrap();

    println!("Hello, world!, {}", &hello);
}
