use std::error::Error;

use config_macro::ConfigTrait;
use goodmorning_bindings::services::v1::{V1Error, V1Response};
use log::*;

use crate::{functions::diritems_tostring, structs::CredsConfig, CREDS, INSTANCE};

use super::duration_as_string;

pub fn ev1_handle(err: &V1Error) -> Result<(), Box<dyn Error>> {
    debug!("Handling error {err:?}");
    match err {
        V1Error::UsernameTaken => println!("The username you've chosen has already been taken by another user,\nplease choose another unique username.\nNote that 2 usernames with different casing are considered the same."),
        V1Error::EmailTaken => println!("This email has already been used for another account,\nyou can only register one account for each email address.\nPlease use your own email address, and stop quit making so many account."),
        V1Error::NoSuchUser => println!("Who... is that?\nBut seriously, this user has not been registered on this instance,\nperhaps you made a mistake."),
        V1Error::PasswordIncorrect => println!("That is not the correct password,\ndouble check if both your password and identifier (email, username, user ID) are correct."),
        V1Error::InvalidToken => {
            let creds = unsafe{ CREDS.get_mut().unwrap() };
            trace!("Token invalid, clearning creds.");
            creds.clear();
            trace!("Writing changes to {:?}", CredsConfig::path());
            creds.save()?;
            println!("The user token you provided is invalid,\nit is likely that someone (hopefully you) has regenerate the token on another device,\nwhich invalidates all existing sessions, including this one.\nPlease run the login command again to gain access to your account.")
        },
        V1Error::NotVerified => println!("Your email address has not been verified,\nthis action requires a verified account."),
        V1Error::InvalidUsername => println!("The username you just provided is invalid,\ndouble check if you've made a typo."),
        V1Error::AlreadyVerified => println!("You have already verified your email address,\nthis action is only available for those with an unverified account."),
        V1Error::Cooldown { remaining } => println!("You are currently on a cooldown of {}.", duration_as_string(*remaining)),
        V1Error::EntryNotFound => println!("The requested publish entry cannot be found,\nthe entry could be removed, or it could just have never been there."),
        V1Error::TimedOut => println!("You action has been timed out,\nit has been running past the duration limit."),
        V1Error::EmailMismatch => println!("The email address you are verifying is not the same as the one currently linked to your account,\nyou might have changed your linkeda email."),
        V1Error::TriggerNotFound => println!("The trigger action you've just tried to run does not exist,\neither the trigger ID is incorrect, or it has been expired."),
        V1Error::PathOccupied => println!("The path you want to operate already exists,\nthis action does not allow you to operate on an already occupied location."),
        V1Error::FileNotFound => println!("The file you've requested cannot be found,\ncheck for typos, perhaps it has been deleted."),
        V1Error::FsError { content } => println!("The server's file system returned an error: {content}"),
        V1Error::FileTooLarge => println!("The file you are trying to upload is too large for the server,\nor you may have exceeded your storage limit."),
        V1Error::NoParent => println!("The specified file path does not have a parent (`../`) component,\nthis is not allowed in the action you are trying to do."),
        V1Error::PermissionDenied => println!("You do not have the permission to perform this action: permission denied"),
        V1Error::TypeMismatch => println!("You tried to run a directory operation on a file, or vice versa.\nMake sure you are doing the right thing to the right object."),
        V1Error::FileTypeMismatch { expected, got } => println!("The file you've just uploaded does not seem to match its extension,\nplease only upload files with the correct extension.\nContent: {expected} | Specified: {got}"),
        V1Error::ExtensionMismatch => println!("The extension between target and source does not match,\nplease to not change the file extension when doing move/copying actions."),
        V1Error::BrowserNotAllowed => println!("Browsers are not allowed to perform this action,\nbut you are not using a browser. What?..."),
        V1Error::JobNotFound => println!("The job ID you've just requested does not exist,\nIt is likely that the job has already been done and removed from queue."),
        V1Error::AlreadyCreated => println!("You account is already enabled for this service,\nsending this request again does nothing at all."),
        V1Error::NotCreated => println!("You have not yet enabled this service for your account,\nplease enable this before using service specific functions."),
        V1Error::TooManyProfileDetails => println!("You have too many profile details,\nplease be a normal person and not a N. Korean officer."),
        V1Error::ExceedsMaximumLength => println!("One of the details have exceeded the maximum length,\ngood luck finding it out."),
        V1Error::BirthCakeConflict => println!("You can only have one of the birthday or cakeday,\nyou don't need to tell us the same thing twice."),
        V1Error::InvalidDetail { index } => println!("You have provided an invalid detail in index {index} (0 based)\nCorrect it or remove it to resolve error."),
        V1Error::GmtOnly => println!("This action is only enabled for GM Tex, and not elsewhere."),
        V1Error::CompileError { content } => println!("There has been a fatal compile error:\n{}", content.lines().map(|s| format!("  {s}")).collect::<Vec<_>>().join("\n")),
        V1Error::InvalidCompileRequest => println!("The compile request you've just sent, is completely invalid.\nPlease make sure the compile target for your format exist for the specified compiler."),
        V1Error::External { content } => println!("An external error occured: {content}"),
        V1Error::FeatureDisabled => println!("This feature is disabled right now,\ntry again later."),
        V1Error::Any { value } => println!("The server responded with a custom response:\n{}", serde_json::to_string(value)?)
    }

    Ok(())
}

pub fn v1_handle(res: &V1Response) -> Result<(), Box<dyn Error>> {
    debug!("Handling response {res:?}");
    #[allow(unused_variables)]
    match res {
        V1Response::Created { id, token } => {
            println!("Account has been created,");
            let creds = unsafe { CREDS.get_mut().unwrap() };
            *creds = CredsConfig {
                id: *id,
                instance: unsafe { INSTANCE.get().unwrap().clone() },
                token: token.clone(),
            };
            trace!("Writing new account creds to {:?}", CredsConfig::path());
            creds.save()?;
            println!("you are now logged in");
        }
        V1Response::Deleted => {
            println!("Account deleted successfully, all info has been irreversibly deleted.");
            let creds = unsafe { CREDS.get_mut().unwrap() };
            creds.clear();
            trace!("Clearning account creads and writing changes to {:?}", CredsConfig::path());
            creds.save()?;
            println!("Login data stored has been deleted.");
        }
        V1Response::Login { id, token } => {
            let creds = unsafe { CREDS.get_mut().unwrap() };
            *creds = CredsConfig {
                id: *id,
                instance: unsafe { INSTANCE.get().unwrap().clone() },
                token: token.clone(),
            };
            trace!("Writing account creds to {:?}", CredsConfig::path());
            creds.save()?;
            println!("you are now logged in");
        }
        V1Response::RegenerateToken { token } => {
            println!("Token regenerated, all other sessions are invalidated.");
            let creds = unsafe { CREDS.get_mut().unwrap() };
            creds.token = token.clone();
            trace!("Writing new account creds to {:?}", CredsConfig::path());
            creds.save()?;
            println!("Except for this device, the new token has been saved.")
        }
        V1Response::Renamed => println!("Account renamed successfully."),
        V1Response::EmailChanged => println!("Email changed successfully, please verify your new email address."),
        V1Response::PasswordChanged => println!("You password has been changed, successfully."),
        V1Response::VerificationSent => println!("A verification email has been sent to your email address,\nplease click the verify link to verify your account."),
        V1Response::Tree { content } => todo!(),
        V1Response::Jobs { current, queue } => todo!(),
        V1Response::Unqueued => println!("Job has been unqueued."),
        V1Response::Triggered => println!("Trigger event has been ran."),
        V1Response::Revoked => println!("Trigger revoked."),
        V1Response::Overwritten => println!("File overwritten successfully."),
        V1Response::DirContent { content } => println!("{}", diritems_tostring(content)),
        V1Response::VisibilityChanged => println!("Visibility changed successfully."),
        V1Response::FileItemCreated => println!("File item created successfully."),
        V1Response::FileItemDeleted => println!("File item deleted successfully."),
        V1Response::Copied => println!("File item copied successfully."),
        V1Response::Moved => println!("File item moved successfully."),
        V1Response::Exists { value } if *value => println!("The requested file item does exist."),
        V1Response::Exists { value } => println!("The requested file item does not exist."),
        V1Response::ServiceCreated => println!("Service has been enabled for account successfully."),
        V1Response::ProfileUpdated => println!("Profile details updated successfully."),
        V1Response::Profile { profile, account } => todo!(),
        V1Response::ProfileOnly { profile } => todo!(),
        V1Response::PfpReset => println!("Profile details has been reset successfully."),
        V1Response::TexCompiled { id, newpath } => todo!(),
        V1Response::TexPublished { id } => todo!(),
        V1Response::TexUserPublish { value } => todo!(),
        V1Response::TexUserPublishes { items } => todo!(),
        V1Response::NothingChanged => println!("Operation returned no errors, but nothing has been changed."),
        V1Response::Error { kind } => return ev1_handle(kind),
        V1Response::Any { value } => println!("The server responded with a custom response:\n{}", serde_json::to_string(value)?)
    }

    Ok(())
}
