pub fn read_pw_confirm() -> String {
    loop {
        let password1 = rpassword::prompt_password("Your password: ").unwrap();

        if password1.len() < 8 {
            println!("Your password is too short, use password of at least 8 characters for better security.");
            continue;
        }

        let password2 = rpassword::prompt_password("Confirm password: ").unwrap();
        if password1 == password2 {
            break password1;
        }

        println!("Password mismatch, please re-enter password.");
    }
}
