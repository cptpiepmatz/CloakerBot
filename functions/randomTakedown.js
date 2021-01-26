"use strict";

const queue = require("queue");

const takedown = require("./takedown");
const readDatabase = require("./readDatabase");
const updateTakedownDate = require("./updateTakedownDate");

const collectionName = "random_takedowns";

/**
 * Randomly pushes the cloaker into a channel to kick somebody.
 * <p> This function is just a starter for the interval.
 * @param {Client} client
 */
function randomTakedown(client) {
  console.log("Initialize random takedowns");

  let kickQueue = queue({autostart: true});

  setInterval(async () => {
    let guildData = await getGuildsToTakedown();
    for (let guildDate of guildData) {
      kickQueue.push(async () => {
        let guild = await client.guilds.cache.get(guildDate.GuildID);
        if (!guild) return;
        console.log("Random Takedown: " + guild.name);
        let voices = guild.voiceStates;
        if (voices.cache.size >= 1) {
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

        await updateTakedownDate(guild, guildDate);
      });
    }
  }, 120000);

  async function getGuildsToTakedown() {
    return await readDatabase(
      collectionName,
      "all",
      {
        doRandomTakedowns: true,
        takedownDate: {
          $lte: new Date()
        }
      }
    );
  }
}

module.exports = randomTakedown;