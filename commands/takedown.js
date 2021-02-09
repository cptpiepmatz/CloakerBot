const takedown = require("../functions/takedown");
const getRandom = require("../functions/getRandom");

const replies = require("../server/replies");
const ownerID = require("../server/ownerID");

/**
 * Commands the bot to perform a takedown.
 * @param {Message} message the message that called the command
 * @returns {Promise<void>}
 */
async function takedownCommand(message) {
  console.log(process._getActiveHandles().length , " in command");
  if (!(message.member.hasPermission("MOVE_MEMBERS")
    || message.author.id === ownerID)) {
    try {
      await message.channel.send(getRandom(replies.noPerms));
    }
    catch (e) {
      console.error(e);

      await message.author.send(getRandom(replies.noPerms));
    }

    return;
  }

  console.log(process._getActiveHandles().length , " command: after perm check");
  // Trying to get the user to take down
  let member = message.mentions.members.first();
  if (!member) {
    let maybeID = message.content.replace("/cloaker", "").trim();
    member = message.guild.members.cache.get(maybeID);
    if (!member) {
      try {
        await message.channel.send(getRandom(replies.noMember));
      }
      catch (e) {
        console.error(e);

        await message.author.send(getRandom(replies.noMember));
      }

      return;
    }
  }

  console.log(process._getActiveHandles().length , " command: after user check");
  // Trying to get the voice channel
  let channel = member.voice.channel;
  if (!channel) {
    try {
      await message.channel.send(getRandom(replies.noChannel));
    }
    catch (e) {
      console.error(e);

      await message.author.send(getRandom(replies.noChannel));
    }

    return;
  }

  console.log(process._getActiveHandles().length , " command: after channel check");
  // Trying to connect
  let connection;
  try {
    connection = await channel.join();
  }
  catch (e) {
    console.error(e);

    try {
      await message.channel.send(getRandom(replies.cantJoin));
    }
    catch (e) {
      console.error(e);

      await message.author.send(getRandom(replies.cantJoin));
    }

    return;
  }

  console.log(process._getActiveHandles().length , " before takedown");
  try {
    await takedown(member, connection);
  }
  catch (e) {
    console.error(e);

    try {
      await message.channel.send(getRandom(replies.failedTakedown));
    }
    catch (e) {
      console.error(e);

      await message.author.send(getRandom(replies.failedTakedown));
    }
  }
}

module.exports = takedownCommand;