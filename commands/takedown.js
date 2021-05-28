"use strict";

const takedown = require("../functions/takedown");
const getRandom = require("../functions/getRandom");
const incStat = require("../functions/incStat.js");
const readDatabase = require("../functions/readDatabase");

const replies = require("../server/replies");
const ownerID = require("../server/ownerID");

/**
 * Commands the bot to perform a takedown.
 * @param {Message} message the message that called the command
 * @returns {Promise<void>}
 */
async function takedownCommand(message) {
  console.log(process._getActiveHandles().length , " in command");

  // Check if the user has the role to take down people
  let dbRoleData = await readDatabase("roles", message.guild, {});
  let allowedRoleId = dbRoleData?.takedownRole;
  let inRole = await message.member.roles.cache.get(allowedRoleId);

  if (!(message.member.hasPermission("MOVE_MEMBERS")
    || message.author.id === ownerID
    || inRole)) {
    try {
      await message.channel.send(getRandom(replies.noPerms));
    }
    catch (e) {
      console.error(e);

      await message.author.send(getRandom(replies.noPerms));
    }

  /**
   * Subroutine to check if command is executable
   *
   * @param {boolean|object} check
   * @param {string[]} replyArray
   * @returns {Promise<boolean>}
   */
  async function checkReply(check, replyArray) {
    if (!check) {
      try {
        await message.channel.send(getRandom(replyArray));
      }
      catch (e) {
        console.error(e);

        await message.author.send(getRandom(replyArray));
      }

      incStat(
        incStat.STATS.TIMES_DENIED_COMMAND_TAKEDOWN,
        message.guild,
        message.member
      ).catch(console.error);

      return false;
    }
    return true;
  }

  if (!await checkReply((message.member.hasPermission("MOVE_MEMBERS")
    || message.author.id === ownerID), replies.noPerms)) return;
  console.log(process._getActiveHandles().length , " command: after perm check");


  // Trying to get the user to take down
  let member = message.mentions.members.first();
  if (!member) {
    let maybeID = message.content.replace("/cloaker", "").trim();
    member = message.guild.members.cache.get(maybeID);
  }
  if (!await checkReply(member, replies.noMember)) return;
  console.log(process._getActiveHandles().length , " command: after user check");

  // Trying to get the voice channel
  let channel = member.voice.channel;
  if (!await checkReply(channel, replies.noChannel)) return;
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

    incStat(incStat.STATS.TAKEDOWNS_FAILED, message.guild)
      .catch(console.error);

    return;
  }

  console.log(process._getActiveHandles().length , " before takedown");
  try {
    if (await takedown(member, connection)) {
      incStat(
        incStat.STATS.TIMES_COMMANDED_TAKEDOWN,
        message.guild,
        message.member
      ).catch(console.error)
    }
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
