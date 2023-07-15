use rand::Rng;
use toml::Table;

// pub static REPLIES: Table = toml::toml!(include_str!())

pub fn select<'r>(replies: &[&'r str]) -> &'r str {
    let index = rand::thread_rng().gen_range(0..replies.len());
    replies[index]
}

// Replies when the bot does do random takedowns
pub const DO_RANDOM: [&str; 3] = [
    "I chokeslam you if you don't see it coming!",
    "Prepare yourself, I'll come in the night",
    "Listen here, I've got you! Don't know when."
];

// Replies when the bot does NOT do random takedowns
pub const DO_NOT_RANDOM: [&str; 3] = [
    "Your little butt seems safe for now.",
    "I only come up for calls!",
    "I've got better to do currently!"
];

// Replies when the user doesn't input a correct statement
pub const INCORRECT_INPUT: [&str; 2] = ["Dude, are you crazy?", "Better get your brain together!"];

// Replies when the takedown was successful
pub const TAKEDOWN_SUCCESSFUL: [&str; 6] = [
    "Ladies and Gentlemen, we got him!",
    "Got what he deserved!",
    "Another day at the office.",
    "He won't leave his safehouse anymore.",
    "Haha, got him.",
    "I think I'll have a protein drink and hit the shower."
];

// Replies when the database has no data for the guild
pub const UNKNOWN_INPUT: [&str; 2] = [
    "Looks like I've never seen you!",
    "What did you mean? Idiot!"
];

// Replies if the user commanding doesn't have the rights to move members
pub const NO_PERMS: [&str; 4] = [
    "Don't tell me what to do!",
    "Shut up you little shit!",
    "You filthy casual. I don't care!",
    "Remember what professor oak told you about bikes!"
];

// Replies if the bot can't find the member
pub const NO_MEMBER: [&str; 3] = [
    "Who the hell is even that!",
    "Who?! You dipshit!",
    "Tell me who, idiot!"
];

// Replies when the member is an invalid target
pub const INVALID_MEMBER: [&str; 2] = ["Fuck no! Not this one!", "That is not our target dipshit!"];

// Replies if the bot can't find the channel to join
pub const NO_CHANNEL: [&str; 5] = [
    "Where you wanted me to go?! Dickhead!",
    "You idiot, he's not there!",
    "I expected you know where he is!",
    "Next time i chokeslam you if you don't call for the right guy!",
    "He's not there! You had one job and even failed that!"
];

// Replies if the bot can't join the channel
pub const CANT_JOIN: [&str; 3] = [
    "I can't go there! Idiot!",
    "Tell me somewhere else to go!",
    "Not even I can get in there. Sorry amigo."
];

// Replies if the bot fails to take down a member
pub const FAILED_TAKEDOWN: [&str; 4] = [
    "Takedown failed! Retreat!",
    "He shot me down!",
    "I can't!",
    "Mission failed, we'll get em next time!"
];

// Replies if the user inputs not a number for takedown chances
pub const NO_NUMBER: [&str; 7] = [
    "You dipshit, give me a number!",
    "That's no number! Idiot!",
    "You absolute clown, give me a number",
    "Jesus, I'm not here to do algebra. Give me a number, asshole!",
    "I'm gonna give you to the count of now to give me a number, before I kick your ass!",
    "Numbers aren't that hard!",
    "I can’t read your mind, give me a damn number!"
];

// Replies if the user inputs the bot itself as the target
pub const COMMAND_TAKEDOWN_SELF: [&str; 3] = [
    "Ah, you think you're smart. Dumpfuck!",
    "I'm not an idiot!",
    "Suck my ass!"
];
