use log::*;

pub fn read_pw_confirm() -> String {
    loop {
        trace!("Reading password with prompt + confirm [1/2]");
        let password1 = rpassword::prompt_password("Your new password: ").unwrap();

        if password1.len() < 8 {
            debug!("Password length is less than 8.");
            println!("Your password is too short, use password of at least 8 characters for better security.");
            continue;
        }

        trace!("Reading password with prompt + confirm [2/2]");
        let password2 = rpassword::prompt_password("Confirm password: ").unwrap();
        if password1 == password2 {
            trace!("Password matches, continuing.");
            break password1;
        }

        println!("Password mismatch, please re-enter password.");
    }
}

pub fn read_pw() -> String {
    trace!("Reading password with prompt.");
    rpassword::prompt_password("Your password: ").unwrap()
}

pub fn read_pw_old() -> String {
    trace!("Reading old password with prompt.");
    rpassword::prompt_password("Your current password: ").unwrap()
}
