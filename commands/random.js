"use strict";

const getRandom = require("../functions/getRandom");
const writeDatabase = require("../functions/database/writeDatabase");
const readDatabase = require("../functions/database/readDatabase");
const clearDatabase = require("../functions/database/clearDatabase.js");
const updateTakedownDate = require("../functions/updateTakedownDate");
const incStat = require("../functions/incStat");

const replies = require("../server/replies");
const ownerId = require("../server/ownerId");

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
      await updateTakedownDate(message.guild);
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
    case "min":
      // set the minimum time for random takedown checks
      let minNumber = Number.parseFloat(args[3]);
      if (Number.isNaN(minNumber)) {
        await message.channel.send(getRandom(replies.noNumber));
        break;
      }
      if (!await permTest(message)) return;
      await writeDatabase(
        collectionName,
        message.guild,
        {
          doRandomTakedowns: true,
          randomTakedownMin: minNumber}
      );
      await updateTakedownDate(message.guild);
      await message.channel.send(
        "*The minimum time between random takedown checks was set to "
        + minNumber
        + " hours.*");
      await message.channel.send(getRandom(replies.doRandom));
      break;
    case "max":
      // set the maximum time for random takedown checks
      let maxNumber = Number.parseFloat(args[3]);
      if (Number.isNaN(maxNumber)) {
        await message.channel.send(getRandom(replies.noNumber));
        break;
      }
      if (!await permTest(message)) return;
      await writeDatabase(
        collectionName,
        message.guild,
        {
          doRandomTakedowns: true,
          randomTakedownMax: maxNumber}
      );
      await updateTakedownDate(message.guild);
      await message.channel.send(
        "*The maximum time between random takedown checks was set to "
        + maxNumber
        + " hours.*");
      await message.channel.send(getRandom(replies.doRandom));
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
        let minNumber = dbData.randomTakedownMin;
        let maxNumber = dbData.randomTakedownMax;
        if (typeof minNumber !== "number") {
          minNumber = "4 (default)";
        }
        if (typeof maxNumber !== "number") {
          maxNumber = "24 (default)";
        }
        await message.channel.send(
          "*Currently the random takedown checks takes between "
          + minNumber
          + " and "
          + maxNumber
          + " hours.*");
        await message.channel.send(getRandom(replies.doRandom));
      }
      else {
        await message.channel.send(getRandom(replies.doNotRandom));
      }
      break;
    default:
      // some weird input
      await message.channel.send(getRandom(replies.incorrectInput));
      incStat(incStat.STATS.TIMES_INCORRECT_COMMAND, message.guild, message.member)
        .catch(console.error);
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
  if (message.author.id === ownerId) return true;
  await message.channel.send(getRandom(replies.noPerms));
  return false;
}

module.exports = randomCommand;
