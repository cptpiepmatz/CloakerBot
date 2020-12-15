const sleep = require("sleep-promise");

const takedown = require("./takedown");

/**
 * Randomly pushes the cloaker into a channel to kick somebody.
 * @param {Client} client
 * @returns {Promise<void>}
 */
async function randomTakedown(client) {
  for (let guild of client.guilds.cache.array()) {
    guild = await guild.fetch();
    let voices = guild.voiceStates;
    if (voices.cache.size > 3) {
      try {
        console.log(voices.cache);
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