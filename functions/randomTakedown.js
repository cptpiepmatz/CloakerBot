const sleep = require("sleep-promise");

const takedown = require("./takedown");
const readDatabase = require("./readDatabase");

const collectionName = "random_takedowns";

/**
 * Randomly pushes the cloaker into a channel to kick somebody.
 * @param {Client} client
 * @returns {Promise<void>}
 */
async function randomTakedown(client) {
  console.log("Starting to random takedown now!");

  let dbData = await readDatabase(
    collectionName,
    "all",
    {doRandomTakedowns: true}
  );
  let randomGuildIDs = [];
  for (let dbDate of dbData) {
    randomGuildIDs.push(dbDate.GuildID);
  }

  let allGuilds = client.guilds.cache.array();
  let filteredGuilds = allGuilds.filter(guild => randomGuildIDs.includes(guild.id));

  for (let guild of filteredGuilds) {
    guild = await guild.fetch();
    let voices = guild.voiceStates;
    if (voices.cache.size > 3) {
      try {
        let member;
        do {
          member = voices.cache.random().member
        }
        while (member.id === client.user.id);
        await takedown(member);
      }
      catch (e) {
        console.error(e);
      }
    }
  }

  // Have some rest. Wait up to 20 hours.
  await sleep(Math.floor(Math.random() * 72000000));
}

module.exports = randomTakedown;