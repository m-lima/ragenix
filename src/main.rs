fn main() {
    let (file, key) = {
        let mut args = std::env::args().skip(1);
        let file = args
            .next()
            .map(std::ffi::CString::new)
            .expect("First parameter should be the file to decrypt")
            .expect("Could not create null-terminated string for the first parameter");
        let key = args
            .next()
            .map(std::ffi::CString::new)
            .expect("Second parameter should be the key to use for decryption")
            .expect("Could not create null-terminated string for the second parameter");
        (file, key)
    };

    let mut code = 0;
    let now = std::time::Instant::now();
    let out = ragenix::decrypt(file.as_ptr(), key.as_ptr(), &mut code);
    let elapsed = now.elapsed();
    if code == 0 {
        println!("[32m[Ok][m {out}");
    } else {
        eprintln!("[31m[Err][m {out}");
    }
    println!("Elapsed: {elapsed:?}");
}
