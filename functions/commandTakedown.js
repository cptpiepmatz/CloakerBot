const takedown = require("./takedown");
const getRandom = require("./getRandom");

// Replies if the user commanding doesn't have the rights to move members
const replyNoPerms = [
  "Don't tell me what to do!",
  "Shut up you little shit!",
  "You filthy casual. I don't care!"
]

// Replies if the bot can't find the member
const replyNoMember = [
  "Who the hell is even that!",
  "Who?! You dipshit!",
  "Tell me who, idiot!"
];

// Replies if the bot can't find the channel to join
const replyNoChannel = [
  "Where you wanted me to go?! Dickhead!",
  "You idiot, he's not there!",
  "I expected you know where he is!",
  "Next time i chokeslam you if you don't call for the right guy!",
  "He's not there! You had one job and even failed that!"
];

// Replies if the bot can't join the channel
const replyCantJoin = [
  "I can't go there! Idiot!",
  "Tell me somewhere else to go!",
  "Not even I can get in there. Sorry amigo."
];

// Replies if the bot fails to takedown a member
const replyFailedTakedown = [
  "Takedown failed! Retreat!",
  "He shot me down!",
  "I can't!"
];

/**
 * Commands the bot to perform a takedown.
 * @param {Message} message
 * @returns {Promise<void>}
 */
async function commandTakedown(message) {
  if (!(message.member.hasPermission("MOVE_MEMBERS")
    || message.author.id === "137280289708834816")) {
    try {
      await message.channel.send(getRandom(replyNoPerms));
    }
    catch (e) {
      console.error(e);

      await message.author.send(getRandom(replyNoPerms));
    }

    return;
  }

  // Trying to get the user to take down
  let member = message.mentions.members.first();
  if (message.content.split(/\s+/)[1].toLowerCase() === "random") {
    member = message.guild.voiceStates.cache.random().member;
  }
  if (!member) {
    try {
      await message.channel.send(getRandom(replyNoMember));
    }
    catch (e) {
      console.error(e);

      await message.author.send(getRandom(replyNoMember));
    }

    return;
  }

  // Trying to get the voice channel
  let channel = member.voice.channel;
  if (!channel) {
    try {
      await message.channel.send(getRandom(replyNoChannel));
    }
    catch (e) {
      console.error(e);

      await message.author.send(getRandom(replyNoChannel));
    }

    return;
  }

  // Trying to connect
  let connection;
  try {
    connection = await channel.join();
  }
  catch (e) {
    console.error(e);

    try {
      await message.channel.send(getRandom(replyCantJoin));
    }
    catch (e) {
      console.error(e);

      await message.author.send(getRandom(replyCantJoin));
    }

    return;
  }

  try {
    await takedown(member, connection);
  }
  catch (e) {
    console.error(e);

    try {
      await message.channel.send(getRandom(replyFailedTakedown));
    }
    catch (e) {
      console.error(e);

      await message.author.send(getRandom(replyFailedTakedown));
    }
  }
}

module.exports = commandTakedown;