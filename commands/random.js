"use strict";

const getRandom = require("../functions/getRandom");
const writeDatabase = require("../functions/writeDatabase");
const readDatabase = require("../functions/readDatabase");
const clearDatabase = require("../functions/clearDatabase.js");

const replies = require("../server/replies");

/**
 * Command to gather infos for random takedowns or set the option.
 *
 * @param {Message} message
 * @returns {Promise<void>}
 */
async function randomCommand(message) {

  let args = message.content.trim().toLowerCase().split(/\s+/);
  const collectionName = "random_takedowns";

  switch (args[2]) {
    case "true":
      // allow random takedowns
      if (!await permTest(message)) return;
      await writeDatabase(
        collectionName,
        message.guild,
        {doRandomTakedowns: true}
      );
      await message.channel.send(
        "*CloakerBot will now do random takedowns.*");
      await message.channel.send(getRandom(replies.doRandom));
      break;
    case "false":
      // disallows random takedowns
      if (!await permTest(message)) return;
      await writeDatabase(
        collectionName,
        message.guild,
        {doRandomTakedowns: false}
      );
      await message.channel.send(
        "*CloakerBot will now stop doing random takedowns.*");
      await message.channel.send(getRandom(replies.doNotRandom));
      break;
    case "default":
      // removes takedown data from database
      if (!await permTest(message)) return;
      await clearDatabase(
        collectionName,
        message.guild
      );
      await message.channel.send(
        "*CloakerBot has now removed this guild from it's database.*");
      break;
    case undefined:
      // replies with the current state
      let dbData = await readDatabase(
        collectionName,
        message.guild
      );
      if (!dbData) {
        await message.channel.send(getRandom(replies.unknownInput));
      }
      else if (dbData.doRandomTakedowns) {
        await message.channel.send(getRandom(replies.doRandom));
      }
      else {
        await message.channel.send(getRandom(replies.doNotRandom));
      }
      break;
    default:
      // some weird input
      await message.channel.send(getRandom(replies.incorrectInput));
  }
}

/**
 * Tests if the member has the needed perms and if not calls out for it
 *
 * @param {Message} message
 * @returns {Promise<boolean>}
 */
async function permTest(message) {
  let hasPerms = message.member.hasPermission("ADMINISTRATOR");
  if (hasPerms) return true;
  await message.channel.send(getRandom(replies.noPerms));
  return false;
}

module.exports = randomCommand;