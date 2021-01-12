"use strict";

const getRandom = require("../functions/getRandom");
const writeDatabase = require("../functions/writeDatabase");
const readDatabase = require("../functions/readDatabase");
const clearDatabase = require("../functions/clearDatabase.js");

// Replies when the bot does do random takedowns
const replyDoRandom = [
  "I chokeslam you if you don't see it coming!",
  "Prepare yourself, I'll come in the night",
  "Listen here, I've got you! Don't know when."
];

// Replies when the bot does NOT do random takedowns
const replyDoNotRandom = [
  "Your little butt seems safe for now.",
  "I only come up for calls!",
  "I've got better to do currently!"
];

// Replies when the user doesn't input a correct statement
const replyIncorrect = [
  "Dude, are you crazy?",
  "Better get your brain together!"
];

// Replies when the database has no data for the guild
const replyUnknown = [
  "Looks like I've never seen you!"
]

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
      await writeDatabase(
        collectionName,
        message.guild,
        {doRandomTakedowns: true}
      );
      await message.channel.send(
        "*CloakerBot will now do random takedowns.*");
      await message.channel.send(getRandom(replyDoRandom));
      break;
    case "false":
      // disallows random takedowns
      await writeDatabase(
        collectionName,
        message.guild,
        {doRandomTakedowns: false}
      );
      await message.channel.send(
        "*CloakerBot will now stop doing random takedowns.*");
      await message.channel.send(getRandom(replyDoNotRandom));
      break;
    case "default":
      // removes takedown data from database
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
        await message.channel.send(getRandom(replyUnknown));
      }
      else if (dbData.doRandomTakedowns) {
        await message.channel.send(getRandom(replyDoRandom));
      }
      else {
        await message.channel.send(getRandom(replyDoNotRandom));
      }
      break;
    default:
      // some weird input
      await message.channel.send(getRandom(replyIncorrect));
  }
}

module.exports = randomCommand;